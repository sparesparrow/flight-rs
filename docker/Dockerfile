FROM rust:1.74-bullseye as builder

# Install Python and dependencies
RUN apt-get update && apt-get install -y \
    python3 \
    python3-pip \
    python3-venv \
    libssl-dev \
    pkg-config

# Create a non-root user for better security
RUN useradd -m -u 1000 dev

# Set up the working directory
WORKDIR /app
RUN chown -R dev:dev /app

# Switch to non-root user
USER dev

# Copy Cargo.toml and cargo lock first to leverage Docker caching
COPY --chown=dev:dev Cargo.toml /app/

# Create a dummy main.rs file to build dependencies
RUN mkdir -p /app/src
RUN echo "fn main() {}" > /app/src/main.rs
RUN echo "fn main() {}" > /app/src/server.rs
RUN cargo build --release

# Setup Python environment
RUN python3 -m venv /app/venv
ENV PATH="/app/venv/bin:$PATH"

# Copy pyproject.toml to install Python dependencies
COPY --chown=dev:dev python/pyproject.toml /app/python/
WORKDIR /app/python
RUN pip install --upgrade pip && \
    pip install poetry && \
    poetry config virtualenvs.create false && \
    poetry install

# Go back to app root
WORKDIR /app

# Now copy the actual source code
COPY --chown=dev:dev . /app/

# Clean the dummy build and rebuild with actual code
RUN cargo clean && cargo build --release

# Expose the server port
EXPOSE 8080

# Create a helper script to run tests or start the server
RUN echo '#!/bin/bash\n\
if [ "$1" = "test" ]; then\n\
  cd /app/python && python -m pytest -xvs\n\
elif [ "$1" = "server" ]; then\n\
  cd /app && cargo run --release --bin server\n\
elif [ "$1" = "standalone" ]; then\n\
  cd /app && cargo run --release --bin standalone\n\
else\n\
  echo "Usage: $0 [test|server|standalone]"\n\
  exit 1\n\
fi' > /app/entrypoint.sh && chmod +x /app/entrypoint.sh

ENTRYPOINT ["/app/entrypoint.sh"]
CMD ["server"] 