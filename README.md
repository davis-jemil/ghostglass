<div align="center">

# GHOSTGLASS PROTOCOL

**Cyber Deception & Active Defense System — Built by Lumin Group**

![Rust](https://img.shields.io/badge/Rust-000000?style=for-the-badge&logo=rust&logoColor=white)
![License](https://img.shields.io/badge/License-MIT-00AEEF?style=for-the-badge)
![Status](https://img.shields.io/badge/Status-Active%20Development-success?style=for-the-badge)

</div>

---

## Overview

Ghostglass Protocol is an active-defense system that turns an attacker's reconnaissance against them. Instead of simply blocking or alerting on intrusion attempts, it absorbs them into a procedurally generated decoy environment — a fake filesystem, a fake shell, fake credentials, and fake services — all indistinguishable from a real production host. Every command, connection, and honeytoken hit is logged, scored, and surfaced live, turning a would-be breach into a fully observed, zero-risk intelligence-gathering exercise. The problem it solves: by the time most security teams notice an intruder, the intruder has already learned something true. Ghostglass ensures they never do.

## How It Works

When a connection lands on Ghostglass, it never touches anything real. The TLS interceptor terminates the handshake and hands the session to a **gaslighter** — a procedural engine that fabricates a self-consistent filesystem, shell, and credential set on the fly, seeded so the same path always looks the same to the same attacker. There is no bottom to this filesystem and no real secret to find; every "juicy" file the attacker reaches for is a honeytoken that fires a silent alert. Meanwhile, every command, connection, and trigger is logged, scored against a skill-tier model, and streamed to a live dashboard so defenders watch the intrusion unfold in real time instead of reconstructing it after the fact.

## Architecture

| Layer | Description |
|---|---|
| **Layer 1 — Deception Core** | TLS interception with fake SNI, PQC handshake stubs, and a procedurally generated "infinite hallway" filesystem served through a fake shell. |
| **Layer 2 — Attacker Awareness** | Persistent session logging, honeytoken triggers on sensitive file access, and a fake HTTP admin login decoy. |
| **Layer 3 — Attacker Intelligence** | Post-session attacker profiling with skill-tier scoring, fabricated cryptographic entropy injection, and a live terminal threat dashboard. |
| **Layer 4 — Web Dashboard** | A real-time web command center exposing session stats, honeytoken alerts, and threat scoring over a polled JSON API. |

## Features

- 🌀 Ephemeral Polymorphic Decoy Environment
- 🔒 TLS 1.3 transparent proxy with fake SNI
- 🧬 Post-Quantum Cryptography (Kyber-1024, Dilithium-5)
- 🕸️ Infinite hallway procedural filesystem
- 💻 Attacker command recognition & fake output
- 🚨 Honeytoken alerts with skill assessment
- 📊 Live web dashboard (GHOSTGLASS COMMAND CENTER)
- 📝 Session logging & attacker profiling

## Quick Start

```bash
cargo build
cargo run
```

Then open the live command center in your browser:

```
http://127.0.0.1:3000
```

The TLS proxy listens on `127.0.0.1:8443` and the fake admin decoy on `127.0.0.1:8080`. Session logs are written to `logs/session_<timestamp>.log`.

## Built With

- [Rust](https://www.rust-lang.org/)
- [Tokio](https://tokio.rs/)
- [Serde](https://serde.rs/)

## License

Licensed under the **MIT License**.

---

<div align="center">

Built by **Lumin Group** — Monrovia, Liberia

</div>
