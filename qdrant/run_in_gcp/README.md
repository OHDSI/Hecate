# Run Qdrant Embedding Builder on GCP GPU VM

Laptop too slow → spin up a GCP VM with a T4 GPU, run `build.sh`, copy output back.

## 1. Create SSH key

```bash
ssh-keygen -t rsa -b 4096 -f ~/.ssh/gcp_hecate -C "$USERNAME"
```

> Key saved to `~/.ssh/gcp_hecate`. Username taken from `$USERNAME` env var.

## 2. Create VM

```bash
gcloud compute instances create hecate-embedding \
  --project=atlas-development-270609 \
  --zone=europe-west1-b \
  --machine-type=n1-standard-4 \
  --accelerator=type=nvidia-tesla-t4,count=1 \
  --image-family=pytorch-2-9-cu129-ubuntu-2204-nvidia-580 \
  --image-project=deeplearning-platform-release \
  --boot-disk-size=100GB \
  --maintenance-policy=TERMINATE \
  --metadata="ssh-keys=$USERNAME:$(cat ~/.ssh/gcp_hecate.pub)"
```

> T4 costs ~$0.35/hr. Expected run time: ~5-10 min for full vocabulary.

## 3. Push PostgreSQL image to Artifact Registry

```bash
# One-time: create registry
gcloud artifacts repositories create hecate \
  --repository-format=docker \
  --location=europe-west1 \
  --project=atlas-development-270609

# Authenticate docker
gcloud auth configure-docker europe-west1-docker.pkg.dev

# Tag + push
docker tag hecate-postgres \
  europe-west1-docker.pkg.dev/atlas-development-270609/hecate/postgres

docker push \
  europe-west1-docker.pkg.dev/atlas-development-270609/hecate/postgres
```

## 4. Copy qdrant scripts to VM

```bash
gcloud compute scp --recurse ../qdrant hecate-embedding:~/qdrant \
  --zone=europe-west1-b --project=atlas-development-270609 \
  --ssh-key-file=~/.ssh/gcp_hecate
```

## 5. SSH into VM

```bash
gcloud compute ssh hecate-embedding \
  --zone=europe-west1-b \
  --project=atlas-development-270609 \
  --ssh-key-file=~/.ssh/gcp_hecate
```

## 6. Install Docker on VM

```bash
curl -fsSL https://get.docker.com | sudo sh
sudo usermod -aG docker $USER
newgrp docker
sudo apt install -y python3.10-venv
```

## 7. Start PostgreSQL and Qdrant on VM

```bash
# Authenticate docker on VM
gcloud auth configure-docker europe-west1-docker.pkg.dev

# Pull + start PostgreSQL
docker run -d --name hecate-db \
  -p 5432:5432 \
  europe-west1-docker.pkg.dev/atlas-development-270609/hecate/postgres

# Start Qdrant (pulled from Docker Hub)
docker run -d --name qdrant \
  -p 6333:6333 -p 6334:6334 \
  qdrant/qdrant:latest
```

## 8. Run embedding builder

```bash
cd ~/qdrant
cp config.template .env
# edit .env — set PG_HOST=localhost, QDRANT_URL=http://localhost:6333
./build.sh
```

PyTorch will auto-detect the T4 (CUDA) and use it for embeddings.

## 9. Copy results back

```bash
gcloud compute scp --recurse hecate-embedding:~/qdrant/collections ./collections \
  --zone=europe-west1-b --project=atlas-development-270609 \
  --ssh-key-file=~/.ssh/gcp_hecate
```

Then copy `collections/` into `qdrant/collections/` and build Docker images as usual.

## 10. Delete VM when done

```bash
gcloud compute instances delete hecate-embedding --zone=europe-west1-b --project=atlas-development-270609
```
