# Use the official Rust base image with the desired version
FROM rust:1.74.0

# Set the working directory to /app
WORKDIR /app

# Copy the entire project folder into the container
COPY . .

# Install Git
RUN apt-get update && \
    apt-get install -y git

# Build the project
# RUN cargo build

# Expose any required ports (if applicable)
EXPOSE 8080

# Start a bash shell when the container starts
CMD ["/bin/bash"]