# Deployment Guide

1. Build images locally.

```sh
docker build -t postgres:latest ./src/postgres
docker build -t redis:latest ./src/redis
docker build -t crawler:latest ./src/crawler
docker build -t api:latest ./src/api
```

2. Create the namespace.

```sh
kubectl create namespace gse
```

3. Deploy to Kubernetes.

```sh
kubectl apply -f postgres-deployment.yaml
kubectl apply -f redis-deployment.yaml
kubectl apply -f crawler-deployment.yaml
kubectl apply -f api-deployment.yaml
```

