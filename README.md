<h1 align="center"> Soroban Smart Contracts </h1>

<h3 align="center"> Maintainers🛠️ </h3>
<table align="center">
  <tr>
    <td align="center">
      <a href="https://github.com/DiegoB1911">
        <img src="https://github.com/DiegoB1911.png" width="100" style="border-radius:50%" />
      </a>
      <br/>
      <strong>Diego Barquero</strong>
    </td>
    <td align="center">
      <a href="https://github.com/salazarsebas">
        <img src="https://github.com/salazarsebas.png" width="100" style="border-radius:50%" />
      </a>
      <br/>
      <strong>Sebástian Salazar</strong>
    </td>
     <td align="center">
      <a href="https://github.com/aguilar1x">
        <img src="https://github.com/aguilar1x.png" width="100" style="border-radius:50%" />
      </a>
      <br/>
      <strong>Matias Aguilar</strong>
    </td>
    <td align="center">
      <a href="https://github.com/Bran18">
        <img src="https://github.com/Bran18.png" width="100" style="border-radius:50%" />
      </a>
      <br/>
      <strong>Brandon Fernández</strong>
    </td>
  </tr>
</table>




---

## 📖 Table of Contents  

1. [🔧 Prerequisites](#-prerequisites)  
2. [⚙️ Environment Setup](#%EF%B8%8F-environment-setup)  
3. [💰 Wallet Configuration](#-wallet-configuration)  
4. [🛠️ Build & Deployment](#%EF%B8%8F-build--deployment)  
5. [✅ Testing & Execution](#-testing--execution)  
6. [📌 Example](#-example)  
7. [❓ Troubleshooting](#-troubleshooting)  

---

## 🔧 Prerequisites  

Before starting, make sure you have the following dependencies installed:  

### 1️⃣ Install Rust  

- **For Linux/macOS**:  

  ```bash
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
  ```

- **For Windows**:  
  Download and install Rust from [rust-lang.org](https://www.rust-lang.org/).  

- **Add WebAssembly Target**:  

  ```bash
  rustup target add wasm32-unknown-unknown
  ```

### 2️⃣ Install Soroban CLI  

- **Using Cargo**:  

  ```bash
  cargo install --locked soroban-cli
  ```

- **Using Homebrew (macOS, Linux)**:  

  ```bash
  brew install soroban-cli
  ```

---

## ⚙️ Environment Setup  

1. **Clone the repository**  

   ```bash
   git clone https://github.com/your-user/your-repo.git
   cd your-repo/packages/soroban
   ```

2. **Build the contract**  

   ```bash
   soroban build
   ```

3. **Run tests**  

   ```bash
   cargo test
   ```

---

## 💰 Wallet Configuration  

1. **Install a Stellar wallet** (e.g., [Freighter Wallet](https://www.freighter.app/)).  
2. **Create a new wallet** and securely store your secret keys.  
3. **Connect the wallet** to the Stellar testnet.  

---

## 🛠️ Build & Deployment  

1. **Compile the contract in release mode**  

   ```bash
   cargo build --release --target wasm32-unknown-unknown
   ```

2. **Deploy the contract** using Soroban CLI  

   ```bash
   soroban contract deploy --wasm target/wasm32-unknown-unknown/release/your_contract.wasm
   ```

---

## ✅ Testing & Execution  

Run unit tests:  

```bash
cargo test
```

Interact with the deployed contract using Soroban CLI or supported wallet tools.  

---

## 📌 Example  

For a practical example of how to interact with these contracts, check out Stellar’s official documentation on [Smart Contracts](https://developers.stellar.org/docs/build/smart-contracts/getting-started/setup).  

---

## 🤝 Contributing

1. Follow Rust best practices
2. Ensure all tests pass
3. Document your changes
4. Add test cases
5. Submit a PR

## 🔗 Useful Links

- [Soroban Documentation](https://soroban.stellar.org)
- [Rust Documentation](https://doc.rust-lang.org)
- [Stellar Documentation](https://developers.stellar.org)

## 💡 Tips

- Use the Soroban CLI for local development
- Test thoroughly on testnet before mainnet
- Keep contract size optimized
- Monitor gas usage
- Use events for contract state changes


## ❓ Troubleshooting  

If you encounter any issues, try these solutions:  

- **Compilation Errors**: Ensure all dependencies are installed and updated.  
- **Deployment Issues**: Verify you’re connected to the correct network (testnet or mainnet) and that your wallet is properly set up.  
- **Test Failures**: Check detailed error messages from `cargo test` to debug the issue.  

---

This README is based on [Stellar’s official documentation](https://developers.stellar.org/docs/build/smart-contracts/getting-started/setup) .  

🚀 **Happy coding!**
