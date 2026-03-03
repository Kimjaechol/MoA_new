import Foundation
import SwiftUI

/// Manages the ZeroClaw agent lifecycle and chat messages.
///
/// Architecture:
/// - On iOS, ZeroClaw runs in-process as a static library (zeroclaw_ios.a).
/// - All messages go to the local gateway at 127.0.0.1:3000.
/// - No messages are sent to external relay servers.
@MainActor
class AgentManager: ObservableObject {
    @Published var status: AgentStatus = .stopped
    @Published var messages: [ChatMessage] = []
    @Published var isThinking = false

    private let gatewayPort: UInt16 = 3000
    private var gatewayUrl: String { "http://127.0.0.1:\(gatewayPort)" }

    enum AgentStatus {
        case stopped, starting, running, thinking, error
    }

    // MARK: - Lifecycle

    /// Start the ZeroClaw gateway in-process.
    func start(provider: String, apiKey: String?) {
        guard status == .stopped || status == .error else { return }
        status = .starting

        Task.detached { [weak self] in
            guard let self else { return }
            let dataDir = Self.dataDirectory()

            // Use nested withCString to safely pass C strings without manual alloc/free
            let result = dataDir.withCString { dataDirPtr in
                provider.withCString { providerPtr in
                    if let key = apiKey, !key.isEmpty {
                        return key.withCString { keyPtr in
                            zeroclaw_start(dataDirPtr, providerPtr, keyPtr, 3000)
                        }
                    } else {
                        return zeroclaw_start(dataDirPtr, providerPtr, nil, 3000)
                    }
                }
            }

            if result == 0 {
                // Wait for gateway to be ready
                var ready = false
                for _ in 0..<60 {
                    try? await Task.sleep(nanoseconds: 500_000_000) // 0.5s
                    if await self.healthCheck() {
                        ready = true
                        break
                    }
                }
                await MainActor.run {
                    self.status = ready ? .running : .error
                }
            } else {
                await MainActor.run {
                    self.status = .error
                }
            }
        }
    }

    /// Stop the ZeroClaw gateway.
    func stop() {
        zeroclaw_stop()
        status = .stopped
    }

    // MARK: - Messaging

    /// Send a user message to the local ZeroClaw agent.
    func send(message text: String) {
        let userMessage = ChatMessage(role: .user, content: text)
        messages.append(userMessage)
        isThinking = true

        Task.detached { [weak self] in
            guard let self else { return }
            do {
                let response = try await self.sendToGateway(message: text)
                await MainActor.run {
                    self.messages.append(ChatMessage(role: .assistant, content: response))
                    self.isThinking = false
                }
            } catch {
                // Fallback: try C-FFI direct call
                let response = self.sendViaFFI(message: text)
                await MainActor.run {
                    if let response, !response.isEmpty {
                        self.messages.append(ChatMessage(role: .assistant, content: response))
                    } else {
                        self.messages.append(ChatMessage(role: .error, content: error.localizedDescription))
                    }
                    self.isThinking = false
                }
            }
        }
    }

    /// Clear all messages.
    func clearMessages() {
        messages.removeAll()
    }

    // MARK: - Network (Local Gateway)

    /// Send a message to the local ZeroClaw gateway via HTTP.
    private func sendToGateway(message: String) async throws -> String {
        guard let url = URL(string: "\(gatewayUrl)/webhook") else {
            throw AgentError.requestFailed
        }
        var request = URLRequest(url: url)
        request.httpMethod = "POST"
        request.setValue("application/json", forHTTPHeaderField: "Content-Type")
        request.timeoutInterval = 120

        let body = ["message": message]
        request.httpBody = try JSONSerialization.data(withJSONObject: body)

        let (data, response) = try await URLSession.shared.data(for: request)

        guard let httpResponse = response as? HTTPURLResponse,
              httpResponse.statusCode == 200 else {
            throw AgentError.requestFailed
        }

        guard let json = try JSONSerialization.jsonObject(with: data) as? [String: Any],
              let responseText = json["response"] as? String else {
            throw AgentError.invalidResponse
        }

        return responseText
    }

    /// Send a message via the C-FFI bridge (fallback).
    private nonisolated func sendViaFFI(message: String) -> String? {
        guard let responsePtr = message.withCString({ zeroclaw_send_message($0) }) else {
            return nil
        }

        let response = String(cString: responsePtr)
        zeroclaw_free_string(responsePtr)
        return response
    }

    /// Health check against the local gateway.
    private nonisolated func healthCheck() async -> Bool {
        guard let url = URL(string: "http://127.0.0.1:3000/health") else { return false }
        var request = URLRequest(url: url)
        request.timeoutInterval = 2

        do {
            let (_, response) = try await URLSession.shared.data(for: request)
            return (response as? HTTPURLResponse)?.statusCode == 200
        } catch {
            return false
        }
    }

    // MARK: - Helpers

    private static func dataDirectory() -> String {
        let paths = FileManager.default.urls(for: .documentDirectory, in: .userDomainMask)
        let documentsDir = paths[0].appendingPathComponent("zeroclaw")
        try? FileManager.default.createDirectory(at: documentsDir, withIntermediateDirectories: true)
        return documentsDir.path
    }

    enum AgentError: LocalizedError {
        case requestFailed
        case invalidResponse

        var errorDescription: String? {
            switch self {
            case .requestFailed: return "Failed to reach local AI engine"
            case .invalidResponse: return "Invalid response from AI engine"
            }
        }
    }
}
