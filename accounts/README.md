*Практика до лекції з акаунтами*

---

1. Робота з solana CLI
- Згенерувати ключі
- Еірдропнути
- Зчитати дані акаунту

2. Зчитування даних в Rust
- Зчитати дані з конфігу
- Зчитати дані з .env
- Ініціалізація клієнта
- Зчитати дані акаунту сендера
- Зчитати данні з системного акаунту
- Зчитати данні PDA Metaplex metadata USDC

---
---

**Робота з solana CLI**

1. Встановіть [cli](https://solana.com/docs/intro/installation) та необхідні залежності 

Версії, що використовувались під час лекцій:
- solana-cli 3.1.8
- anchor-0.32.1


2. Перевірте поточний конфіг:

```bash
solana config get
```

3. Перемкніться на девнет:

```bash
solana config set --url d
```

4. Згенеруйте дефолтні ключі (в подальшому будуть використовуватись за замовчуванням):

```bash
solana-keygen new
```

Або визначте файл самостійно:

```bash
solana-keygen new -o <filename>.json
```

5. Зчитайте публічний ключ з конфігу:

```bash
solana address
```

6. Cпробуйте зчитати акаунт:

```bash
solana account <pubkey>
```
(Error: AccountNotFound - його ще не існує в мережі)

Або ж зі створеного файлу:

```bash
solana-keygen pubkey <filename>.json
```

7. Аірдропніть через cli:

```bash
solana airdrop <amount> <address>
```

Або скористайтесь [фаусетом](https://faucet.solana.com/)

8. Перевірте баланс:

```bash
solana balance
```

9. Cпробуйте ще раз зчитати акаунт:

```bash
solana account <pubkey>
```

---
**Зчитування даних в Rust**

1. Збілдіть і запустіть проект
```bash
cargo run -p accounts
```
