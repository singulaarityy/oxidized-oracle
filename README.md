# Veritas Edge: Trusted IoT Oracle 🦀 + 📡

![Rust](https://img.shields.io/badge/Backend-Rust-orange?style=for-the-badge&logo=rust)
![C++](https://img.shields.io/badge/Firmware-C++-blue?style=for-the-badge&logo=c%2B%2B)
![Solana](https://img.shields.io/badge/Network-Solana-purple?style=for-the-badge&logo=solana)

## 📖 Introduction

**Veritas Edge** is a Proof-of-Concept for a Decentralized Physical Infrastructure Network (DePIN). It solves the "Garbage In, Garbage Out" problem in IoT-Blockchain integration.

Instead of trusting the data blindly, this system ensures data integrity by performing **cryptographic signing at the hardware edge**.

### The Problem
In standard IoT systems, a compromised server or a Man-in-the-Middle attack can alter sensor data before it reaches the database.

### Our Solution
1.  **Identity:** The IoT device (ESP32) generates a unique Ed25519 keypair on-chip. The private key never leaves the device.
2.  **Signing:** Every sensor reading is hashed and signed locally by the device.
3.  **Verification:** The Rust backend verifies the signature. If valid, the data hash is committed to the Solana Blockchain as immutable proof.

---

## 🏗️ Architecture

```mermaid
sequenceDiagram
    participant Device as ESP32 (IoT)
    participant Server as Rust Backend
    participant Chain as Solana Devnet
    
    Device->>Device: Read Sensor & Sign Data (Ed25519)
    Device->>Server: Send JSON {Data + Signature}
    Server->>Server: Verify Signature with Public Key
    alt Signature Valid
        Server->>Chain: Submit Data Hash (Memo Transaction)
        Chain-->>Server: Transaction Confirmed
        Server-->>Device: Success (200 OK)
    else Signature Invalid
        Server-->>Device: Reject (401 Unauthorized)
    end
