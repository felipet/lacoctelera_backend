#!/bin/env bash

IMAGE_NAME="lacoctelera_backend"
IMAGE_REPO="ghcr.io/felipet/lacoctelera_backend:main"

# Check if the image needs to get updated
docker pull $IMAGE_REPO | grep "up to date" &>> /dev/null

if [ $? != 0 ]; then
  echo "The container is running an old image"

  docker ps | grep $IMAGE_NAME &>> /dev/null

  if [ $? == 0 ]; then
    name=$(docker ps --format json | grep $IMAGE_NAME | grep -o "Names.:\"[a-z_]*\"" | cut -d":" -f2 | tr -d '"')
   
    docker stop $name
    docker rm $name
    echo "Container deleted. Starting a new container using the latest image"
    docker run -d \
      --restart unless-stopped \
      --network host \
      --env-file /root/lacoctelera.env \
      ghcr.io/felipet/lacoctelera_backend:main

    echo "New image running"
  fi
else
  echo "The container is running the latest image, no need to update"
fi

exit 0
