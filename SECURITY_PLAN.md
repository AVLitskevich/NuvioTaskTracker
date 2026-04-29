# Nuvio Security Improvement Plan: Secret Management

## Current State
The application uses `dotenvy` to load environment variables from a `.env` file and `std::env::var` to retrieve them. This is sufficient for local development but risks committing the `.env` file to version control.

## Proposed Strategy: Moving away from `.env`

### 1. Local Development
Continue using `.env` for local development. Ensure `.env` is listed in `.gitignore` to prevent accidental commits.

### 2. Production (Docker/Kubernetes)
In production environments, we should rely on the orchestrator to inject secrets directly as environment variables, avoiding the need for `dotenvy` or file-based secrets where possible.

*   **Docker Compose**: Use environment variables passed via the shell or an `.env` file that is *not* checked into the repository.
    ```yaml
    # Example
    services:
      bot:
        image: nuvio-bot:latest
        environment:
          - DATABASE_URL=${DATABASE_URL}
          - BOT_TOKEN=${BOT_TOKEN}
    ```
*   **Kubernetes**: Use Kubernetes Secrets to inject environment variables into the pod.
    ```yaml
    # Example pod spec
    env:
      - name: BOT_TOKEN
        valueFrom:
          secretKeyRef:
            name: nuvio-secrets
            key: bot-token
    ```

### 3. Implementation
The current code already reads environment variables using `std::env::var`. The application logic is compatible with this approach. To fully adopt this, simply ensure that the production container environment *does not* provide a `.env` file, and `dotenvy::dotenv().ok()` will safely do nothing, allowing the system environment variables to take precedence.

- **Recommendation**: Ensure `dotenvy` is kept in `Cargo.toml` as a dev-dependency or just let it be, but *ensure* that `dotenvy::dotenv().ok()` is present in `main.rs`. Since it doesn't overwrite existing environment variables, it remains safe.
