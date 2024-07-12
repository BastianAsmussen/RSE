# syntax=docker/dockerfile:1

ARG GO_VERSION=1.22.5
ARG APP_NAME=api

FROM golang:${GO_VERSION}-alpine AS build
ARG APP_NAME

WORKDIR /app

# Install host build dependencies.
RUN apk add --no-cache git

COPY go.mod go.sum ./
RUN go mod download && go mod verify

COPY . .

# Build the Go binary.
RUN CGO_ENABLED=0 GOOS=linux GOARCH=amd64 go build -o /bin/server .

FROM alpine:latest AS final
ARG APP_NAME

# Add a non-root user.
ARG UID=10001
RUN adduser \
    --disabled-password \
    --gecos "" \
    --home "/nonexistent" \
    --shell "/sbin/nologin" \
    --no-create-home \
    --uid "${UID}" \
    appuser
USER appuser

# Copy the Go binary from the build stage.
COPY --from=build /bin/server /bin/

EXPOSE 8080
CMD ["/bin/server"]

