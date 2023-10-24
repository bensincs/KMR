# 🚀 Kafka Multicast Relay (KMR) 🚀

## Introduction

Kafka Multicast Relay, abbreviated as KMR, is a powerful tool designed to replay Kafka messages on specific multicast groups while adhering to topic-to-multicast group mappings. This innovative solution provides seamless integration between Kafka message streams and multicast communication, allowing for efficient distribution of data across networks.

KMS is engineered to operate within any Docker environment, ensuring flexibility and ease of deployment for various use cases. With its intuitive configuration options and robust performance, KMS empowers developers to efficiently manage and relay Kafka messages to multicast groups, facilitating effective data dissemination.

## Table of Contents
- [Goals](#goals)
- [Getting Started](#getting-started)
  - [Prerequisites](#prerequisites)
  - [Installation](#installation)
- [Usage](#usage)
- [Contributing](#contributing)
- [License](#license)
- [Acknowledgments](#acknowledgments)
- [Contact](#contact)

## Goals 🎯

- Relay Kafka messages to multicast groups.
- Support configuration-driven multicast group mapping.
- Multicast Capture and Republish to Kafka.

### Goal 1: Relay Kafka messages to multicast groups 📤
The primary objective of KMS is to efficiently forward Kafka messages to specific multicast groups. This ensures that data originating from Kafka topics is reliably and promptly distributed across designated network segments.

### Goal 2: Support configuration-driven multicast group mapping ⚙️
KMS offers a flexible configuration system that allows users to define mappings between Kafka topics and corresponding multicast groups. This enables fine-grained control over message routing, ensuring that data reaches the intended recipients accurately.

### Goal 3: Multicast Capture and Republish to Kafka 🔄
KMS is designed to actively listen for incoming data from multicast groups. Once captured, this data is efficiently re-published into the Kafka ecosystem, establishing a seamless bidirectional communication channel between Kafka and multicast networks.

## Getting Started

### Prerequisites

- Open the Dev Container

### Installation

Nothing to do here.

## Running With Docker Compose 🐳

-  Just run ```docker compose up``` to stand 3 instances of the KRM and 3 instances of kafka.

## Testing the docker compose deployment 🧪

-  Start the producer using kafka cli ```docker exec -it kmr-kafka1-1 bash``` and then ```/opt/bitnami/kafka/bin/kafka-console-producer.sh --broker-list localhost:9092 --topic tracks```
-  In a different terminal consume data from the same topic on cluster 2 ```docker exec -it kmr-kafka2-1  bash``` and then ```/opt/bitnami/kafka/bin/kafka-console-consumer.sh --bootstrap-server localhost:9092 --topic tracks --from-beginning```
-  In a 3rd terminal consume data from the same topic on cluster 3 ```docker exec -it kmr-kafka3-1  bash``` and then ```/opt/bitnami/kafka/bin/kafka-console-consumer.sh --bootstrap-server localhost:9092 --topic tracks --from-beginning```
- Back in the first terminal, send some data to the topic ```tracks``` by simply typing some data and pressing send. You will see it being consumed on the second terminal.
- This shows that the kafka messages are being replicated across the two kafka clusters correctly.

## Contributing 🤝

We welcome contributions from the community! If you'd like to contribute to KMS, please follow these guidelines:

- [Contributing Guidelines](CONTRIBUTING.md)

## License 📝

Kafka Multicast Relay (KMS) is licensed under the [Apache License 2.0](LICENSE).

## Contact 📬

For any questions, feedback, or inquiries, please reach out to us at:

- Email: [your.email@example.com](mailto:your.email@example.com)
- Twitter: [@YourTwitterHandle](https://twitter.com/YourTwitterHandle)

🚀🚀🚀🚀🚀🚀🚀🚀🚀🚀🚀🚀🚀🚀🚀🚀🚀🚀🚀🚀🚀
