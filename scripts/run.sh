#!/bin/bash

USER_ID=$(id -u)
GROUP_ID=$(id -g)
PWD=$PWD

docker build --network host --build-arg USER_ID=${USER_ID} --build-arg GROUP_ID=${GROUP_ID} -t payments  - < Dockerfile

docker run --network host -v $PWD:/home/payments -it payments:latest bash -c "cargo test;bash"
