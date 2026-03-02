package ai.zeroclaw.android.bridge

import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext

/**
 * Bridge to ZeroClaw Rust library via UniFFI-generated bindings.
 *
 * Architecture: MoA Android launches ZeroClaw as a local process.
 * This bridge communicates with the local gateway via HTTP (127.0.0.1:3000)
 * through the UniFFI-generated ZeroClawController.
 *
 * Native library: libzeroclaw_android.so
 * Build: cargo ndk -t arm64-v8a -o app/src/main/jniLibs build --release -p zeroclaw-android-bridge
 */
object ZeroClawBridge {

    private var controller: uniffi.zeroclaw_android_bridge.ZeroClawController? = null
    private var initialized = false

    /**
     * Initialize the ZeroClaw runtime.
     * Must be called before any other methods.
     */
    fun initialize(dataDir: String): Result<Unit> {
        return runCatching {
            System.loadLibrary("zeroclaw_android")
            controller = uniffi.zeroclaw_android_bridge.ZeroClawController.withDefaults(dataDir)
            initialized = true
        }
    }

    /**
     * Initialize with a specific configuration.
     */
    fun initializeWithConfig(config: ZeroClawConfig): Result<Unit> {
        return runCatching {
            System.loadLibrary("zeroclaw_android")
            controller = uniffi.zeroclaw_android_bridge.ZeroClawController(
                uniffi.zeroclaw_android_bridge.ZeroClawConfig(
                    dataDir = config.dataDir,
                    provider = config.provider,
                    model = config.model,
                    apiKey = config.apiKey,
                    systemPrompt = config.systemPrompt
                )
            )
            initialized = true
        }
    }

    /**
     * Start the ZeroClaw gateway.
     * Launches the zeroclaw binary and waits for it to be ready.
     */
    suspend fun start(configPath: String): Result<Unit> = withContext(Dispatchers.IO) {
        check(initialized) { "ZeroClawBridge not initialized" }
        runCatching {
            controller?.start()
                ?: throw IllegalStateException("Controller not initialized")
        }
    }

    /**
     * Stop the ZeroClaw gateway.
     */
    suspend fun stop(): Result<Unit> = withContext(Dispatchers.IO) {
        runCatching {
            controller?.stop()
        }
    }

    /**
     * Send a message to the agent via the local gateway.
     * Returns the assistant's response.
     */
    suspend fun sendMessage(message: String): Result<String> = withContext(Dispatchers.IO) {
        check(initialized) { "ZeroClawBridge not initialized" }
        runCatching {
            val result = controller?.sendMessage(message)
                ?: throw IllegalStateException("Controller not initialized")
            if (result.success) {
                // Get the latest assistant message from history
                val messages = controller?.getMessages() ?: emptyList()
                messages.lastOrNull { it.role == "assistant" }?.content
                    ?: "No response"
            } else {
                throw RuntimeException(result.error ?: "Unknown error")
            }
        }
    }

    /**
     * Get current agent status.
     */
    fun getStatus(): AgentStatus {
        if (!initialized) return AgentStatus.Stopped
        return when (val status = controller?.getStatus()) {
            is uniffi.zeroclaw_android_bridge.AgentStatus.Stopped -> AgentStatus.Stopped
            is uniffi.zeroclaw_android_bridge.AgentStatus.Starting -> AgentStatus.Starting
            is uniffi.zeroclaw_android_bridge.AgentStatus.Running -> AgentStatus.Running
            is uniffi.zeroclaw_android_bridge.AgentStatus.Thinking -> AgentStatus.Thinking
            is uniffi.zeroclaw_android_bridge.AgentStatus.Error -> AgentStatus.Error
            else -> AgentStatus.Stopped
        }
    }

    /**
     * Check if the local gateway is reachable.
     */
    suspend fun isGatewayRunning(): Boolean = withContext(Dispatchers.IO) {
        controller?.isGatewayRunning() ?: false
    }

    /**
     * Check if the native library is loaded and controller is initialized.
     */
    fun isLoaded(): Boolean = initialized

    /**
     * Get conversation history.
     */
    fun getMessages(): List<ChatMessageData> {
        if (!initialized) return emptyList()
        return controller?.getMessages()?.map { msg ->
            ChatMessageData(
                id = msg.id,
                content = msg.content,
                role = msg.role,
                timestampMs = msg.timestampMs
            )
        } ?: emptyList()
    }

    /**
     * Clear conversation history.
     */
    fun clearMessages() {
        controller?.clearMessages()
    }

    /**
     * Update the provider API key.
     */
    fun updateApiKey(provider: String, apiKey: String): Result<Unit> {
        return runCatching {
            val current = controller?.getConfig()
                ?: throw IllegalStateException("Controller not initialized")
            controller?.updateConfig(
                uniffi.zeroclaw_android_bridge.ZeroClawConfig(
                    dataDir = current.dataDir,
                    provider = provider,
                    model = current.model,
                    apiKey = apiKey,
                    systemPrompt = current.systemPrompt
                )
            )
        }
    }

    /**
     * Check if API key is configured.
     */
    fun isConfigured(): Boolean = controller?.isConfigured() ?: false
}

enum class AgentStatus {
    Stopped,
    Starting,
    Running,
    Thinking,
    Error
}

/**
 * Chat message data class for the UI layer.
 */
data class ChatMessageData(
    val id: String,
    val content: String,
    val role: String,
    val timestampMs: Long
)

/**
 * Configuration for ZeroClaw.
 */
data class ZeroClawConfig(
    val dataDir: String = "",
    val provider: String = "anthropic",
    val model: String = "claude-sonnet-4-5",
    val apiKey: String = "",
    val systemPrompt: String? = null,
    val maxTokens: Int = 4096,
    val temperature: Double = 0.7
)
