# MiniSQL Engine in Rust

A lightweight, disk-backed, zero-dependency custom SQL database engine built from scratch in Rust.

It features a custom SQL parser, full CRUD operations (Create, Read, Update, Delete), binary image (`BLOB`) storage support, persistent file modifications on disk, and an interactive REPL shell.

---

## Features

- **Zero Third-Party Dependencies**: Built exclusively with Rust standard library (`std`).
- **Disk Persistence**: Stores database schemas (`.meta`) and raw records (`.data`) locally in a `./data/` folder.
- **Full CRUD Support**: Execute `CREATE TABLE`, `INSERT`, `SELECT`, `UPDATE`, and `DELETE`.
- **Binary/Image Storage (BLOB)**: Store real image bytes/files directly inside BLOB columns using transparent hex serialization.
- **Interactive CLI REPL**: Execute SQL commands live in a terminal session.

---

## Getting Started

### Prerequisites
Make sure you have Rust and Cargo installed:
```bash
rustc --version
cargo --version