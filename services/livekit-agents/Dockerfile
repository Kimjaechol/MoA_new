# MoA LiveKit Agent Worker
# Runs the voice AI pipeline: Deepgram STT → Gemini LLM → Typecast TTS
#
# Build: docker build -t moa-agent .
# Run:   docker run --env-file .env moa-agent

FROM python:3.12-slim

# System deps for audio processing (av/ffmpeg)
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
        ffmpeg \
        libsndfile1 \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Install Python deps first (cache layer)
COPY requirements.txt .
RUN pip install --no-cache-dir -r requirements.txt

# Download model files (Silero VAD, turn detector)
# This bakes the models into the image so startup is fast
COPY agent_pipeline.py .
RUN python agent_pipeline.py download-files

# Copy all source
COPY . .

# Health check (LiveKit agent exposes HTTP health endpoint)
HEALTHCHECK --interval=30s --timeout=5s --retries=3 \
    CMD python -c "import urllib.request; urllib.request.urlopen('http://localhost:8081/health')" || exit 1

# Start agent in production mode
CMD ["python", "agent_pipeline.py", "start"]
