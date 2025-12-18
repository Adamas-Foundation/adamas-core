# ADAMAS Protocol (ADM)

<div align="center">
  <img src="logo.png" alt="Adamas Logo" width="200" />
  <br />
  <h3>The Weightless Fortress.</h3>
  <p>
    The world's first <b>Post-Quantum Layer-1</b> with Interactive CLI & GossipSub Networking.<br>
    Secure as a bunker. Light as a feather. Eternal by design.
  </p>

  <p>
    <a href="https://rust-lang.org">
      <img src="https://img.shields.io/badge/Built_with-Rust-dca282.svg?style=flat-square" alt="Rust">
    </a>
    <a href="#">
      <img src="https://img.shields.io/badge/Cryptography-Dilithium_5-blueviolet.svg?style=flat-square" alt="Crypto">
    </a>
    <a href="#">
      <img src="https://img.shields.io/badge/Network-P2P_Active-success.svg?style=flat-square" alt="P2P">
    </a>
    <a href="#">
      <img src="https://img.shields.io/badge/Status-Alpha_v0.1.0-orange.svg?style=flat-square" alt="Status">
    </a>
  </p>
</div>

---

## 1. The Manifesto: Solving the Trilemma

The blockchain industry is fragmented:
* **Bitcoin** is secure but slow, energy-intensive, and quantum-vulnerable.
* **Solana** is fast but centralizes power in massive data centers.
* **Ethereum** is versatile but burdened by legacy tech debt and heavy fees.

**Adamas is the Synthesis.**
We are building the first architecture that delivers **Quantum Security** without the data weight, and **High-Speed Programmability** without centralization, powered by a sustainable economic model designed to last into the 22nd century.

---

## 2. Core Innovations (The "Unfair" Advantage)

### A. Ephemeral Quantum Signatures (EQS) ™
*Solving the "Post-Quantum Bloat" problem.*
Standard quantum signatures (like Dilithium) are heavy (5KB+). Storing them bloats the chain.
**The Adamas Solution:**
1.  **Sign:** Users sign transactions off-chain via NIST-standard PQC algorithms (**Active in v0.1.0**).
2.  **Compress:** We utilize **Recursive ZK-STARKs** to compress the heavy signature into a tiny validity proof.
3.  **Store:** Only the tiny proof is stored on the ledger.
*Result: Military-grade security with the data-lightness necessary for global scale.*

### B. The Adamas Virtual Machine (AVM)
*Beyond Payments.*
Adamas is not just a currency; it is a global computer.
* **WASM-Based:** Developers can write high-performance Smart Contracts using Rust, C++, or Go.
* **Quantum-Safe:** The first execution environment resistant to quantum decryption attacks.

---

## 3. Tokenomics: The Golden Ratio Standard

Adamas rejects arbitrary supply caps. Our monetary policy is governed by universal mathematical laws.

* **Ticker:** $ADM
* **Total Supply:** **20,633,239 ADM** (Strict Hard Cap)

This figure corresponds exactly to the **35th Lucas Number ($L_{35}$)**. By adhering to the Lucas sequence (intrinsically linked to the Golden Ratio $\phi$), Adamas is **mathematically scarcer than Bitcoin**, embedding organic perfection directly into the protocol.

---

## 4. Technical Architecture: Status v0.1.0

The current Alpha build implements the core pillars of the Adamas architecture using **Rust**.

### A. The Tech Stack (Implemented)
* **Language:** Rust (Memory safety & performance).
* **Cryptography:** **CRYSTALS-Dilithium Level 5** (Native Implementation). Keys and Signatures are fully Post-Quantum secure.
* **Networking:** **libp2p GossipSub**. Nodes can discover peers, exchange messages, and maintain mesh stability.
* **Storage:** **Sled**. High-performance embedded key-value database for block persistence.
* **Mempool:** In-memory transaction validation and queueing system.

### B. The Tri-Layer Ecosystem (Planned)
1.  **ZK-Provers:** GPU nodes for compression.
2.  **Consensus Validators:** Staking nodes for ordering.
3.  **Mobile Verifiers:** Light clients for decentralization.

---

## 5. 🛠️ Quick Start (Interactive Mode)

Adamas v0.1.0 features a fully interactive **Command Line Interface (CLI)**. You can run a node, generate quantum keys, create transactions, and mine blocks manually.

## 6. Roadmap & Status

* **Phase I: The Foundation (COMPLETED ✅)**
    * [x] Core Architecture in Rust.
    * [x] Lucas Supply Logic ($L_{35}$).
    * [x] **Post-Quantum Cryptography** (Dilithium-5 Integrated).
    * [x] **P2P Networking** (GossipSub Handshake Active).
    * [x] **Database Persistence** (Genesis Block & Chain History).

* **Phase II: The Engine (ACTIVE 🚧)**
    * [x] **Interactive CLI** (Wallet, Mining, Mempool Control).
    * [ ] Automatic Block Propagation (Syncing between nodes).
    * [ ] WASM Virtual Machine (AVM) Prototype.
    * [ ] Private Testnet Launch.

* **Phase III: The Awakening**
    * [ ] ZK-STARK Compression Layer.
    * [ ] Public Incentivized Testnet.
    * [ ] Mobile Verifier App Beta.

---

## 7. Contributing & Developers

Adamas is an open-source protocol built for the next century.
We are actively recruiting core contributors in the following areas:
* **Rust / Systems Programming** (Core Logic)
* **Applied Cryptography** (ZK-STARKs & Lattice-based Signatures)
* **Distributed Systems** (P2P Networking)

> *If you want to build the Post-Quantum future, fork this repo and submit a PR, or contact the Foundation.*

---

## License & Status

* **License:** MIT Open Source.
* **Status:** Alpha v0.1.0 / Active Development.

> *"Mathematics is the only truth that endures. Adamas is built on mathematics.