## 批量修复

```bash
### 删除未使用的依赖项
cargo install cargo-machete && cargo machete
### 格式化
cargo fmt --all && cargo clippy --fix --allow-dirty --allow-staged
```