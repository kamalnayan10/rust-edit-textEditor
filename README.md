# ✍️ Rusty Editor — A Cross-Platform CLI Text Editor

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

**Rusty Editor** is a fast, minimal, and cross-platform text editor built entirely in **Rust**, using the [`crossterm`](https://crates.io/crates/crossterm) crate for terminal interactions. Inspired by tools like **Vim**, it provides a responsive command-line editing experience with the safety and performance of Rust.

---

## 🚀 Features

- 🖋️ **Efficient CLI Interface**  
  Smooth keyboard-driven editing experience with minimal latency.

- 💻 **Cross-Platform Compatibility**  
  Runs consistently on **Linux**, **macOS**, and **Windows** terminals.

- ⚡ **Powered by Crossterm**  
  Uses the `crossterm` crate to manage cursor, input, color, and screen buffers across platforms.

- 🛠️ **Performance-Oriented**  
  Built in Rust for safety, speed, and low memory footprint.

- 🧹 **Clean and Maintainable Codebase**  
  Easy to extend and customize with well-structured modules.

---

## 🔧 Installation

### 1. Install Rust

If you don’t have Rust installed, install it from the official site:  
📎 [https://www.rust-lang.org/tools/install](https://www.rust-lang.org/tools/install)

### 2. Clone the Repository

```bash
git clone https://github.com/kamalnayan10/rust-edit-textEditor
cd rust-edit-textEditor
```

### 3. Build the Project

```bash
cargo build --release
```

### 4. Run the Editor

- To open an existing file:

```bash
cargo run --release -- path/to/your/file.txt
```

- To create or open a new file:

```bash
cargo run --release
```

---

## 🧪 Example

```
> cargo run --release my_notes.txt
```

Edit the file using intuitive keyboard commands — save, delete, insert, and navigate — all within the terminal.

---

## 🤝 Contributing

Contributions are welcome! If you find a bug or have a feature request:

1. Fork the repository
2. Create your branch (`git checkout -b feature/foo`)
3. Commit your changes (`git commit -am 'Add foo feature'`)
4. Push to the branch (`git push origin feature/foo`)
5. Open a pull request

---

## 📄 License

This project is licensed under the MIT License — see the [LICENSE](LICENSE) file for details.

---

## ❤️ Acknowledgments

- Inspired by [Neovim](https://neovim.io/)
- Built with [Rust](https://www.rust-lang.org/) and [Crossterm](https://crates.io/crates/crossterm)

---

**Built for learning, crafted with love.**
