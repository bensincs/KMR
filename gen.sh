#!/bin/bash

for i in {1..1000}
do
  echo "PRODUCER keyed-message-asdas"
  echo "topic: tracks"
  echo "key: some_ship_$i"
  echo "lat: $((RANDOM % 180 - 90)).$((RANDOM % 10000)) long: $((RANDOM % 360 - 180)).$((RANDOM % 10000))"
  echo "###"
done