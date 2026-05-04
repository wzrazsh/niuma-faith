# 牛马信仰 — 构建与还原指南

> 本文档描述如何从零还原整个项目。删除全部代码后，按本文档步骤可重建可运行的应用。

## 1. 环境依赖

### 1.1 必需工具

| 工具 | 版本 | 用途 | 安装命令 |
|------|------|------|----------|
| Node.js | 18+ | 前端构建 | [nodejs.org](https://nodejs.org) |
| npm | 9+ | 包管理 | 随 Node.js 安装 |
| Rust | 1.75+ | 后端编译 | [rustup.rs](https://rustup.rs) |
| Tauri CLI | v2 | 桌面应用构建 | `cargo install tauri-cli --version "^2.0"` |

### 1.2 Windows 特有依赖

- **WebView2 Runtime**: Windows 10/11 通常已预装。Tauri v2 需要它。
- **Visual Studio Build Tools** (可选): 若编译失败，安装 C++ 构建工具

### 1.3 验证安装

```bash
node --version    # v18.x+
npm --version     # 9.x+
rustc --version   # 1.75+
cargo --version   # 1.75+
cargo tauri --version  # tauri-cli 2.x
```

---

## 2. 项目结构还原

按以下目录结构创建项目：

```
niuma-faith/
├── package.json              # 前端依赖 + scripts
├── tsconfig.json             # TypeScript 配置
├── tsconfig.node.json        # Vite 的 TS 配置
├── vite.config.ts            # Vite 构建配置
├── index.html                # HTML 入口
├── frontend/
│   └── src/                  # Vue 3 前端源码
│       ├── main.ts
│       ├── App.vue
│       ├── router.ts
│       ├── style.css
│       ├── vite-env.d.ts
│       ├── api/
│       ├── components/
│       ├── services/
│       ├── stores/
│       ├── types/
│       └── utils/
├── src-tauri/
│   ├── Cargo.toml            # Rust 依赖
│   ├── tauri.conf.json       # Tauri 应用配置
│   ├── build.rs              # Tauri 构建脚本
│   ├── icons/                # 应用图标 (32x32, 128x128, 128x128@2x, icns, ico)
│   ├── capabilities/         # Tauri v2 权限配置
│   └── src/
│       ├── main.rs
│       ├── lib.rs
│       ├── domain/
│       ├── data/
│       ├── application/
│       └── tauri/
└── docs/                     # 文档（本文档所在）
```

---

## 3. 配置文件还原

### 3.1 package.json

```json
{
  "name": "niuma-faith",
  "version": "0.1.0",
  "type": "module",
  "scripts": {
    "dev": "vite",
    "build": "vue-tsc --noEmit && vite build",
    "preview": "vite preview",
    "tauri": "tauri"
  },
  "dependencies": {
    "@tauri-apps/api": "^2.0.0",
    "@tauri-apps/plugin-store": "^2.0.0",
    "pinia": "^2.1.7",
    "vue": "^3.4.21",
    "vue-router": "^4.6.4"
  },
  "devDependencies": {
    "@tauri-apps/cli": "^2.0.0",
    "@vitejs/plugin-vue": "^5.0.4",
    "playwright": "^1.59.1",
    "typescript": "~5.4.0",
    "vite": "^5.2.0",
    "vue-tsc": "^2.0.6"
  }
}
```

### 3.2 vite.config.ts

```typescript
import { defineConfig } from 'vite';
import vue from '@vitejs/plugin-vue';
import { resolve } from 'path';

export default defineConfig({
  plugins: [vue()],
  resolve: {
    alias: {
      '@': resolve(__dirname, 'frontend/src'),
    },
  },
  build: {
    outDir: 'frontend/dist',
  },
  server: {
    port: 5173,
    strictPort: true,
    watch: {
      ignored: ['**/src-tauri/**'],
    },
  },
  clearScreen: false,
});
```

### 3.3 tsconfig.json

```json
{
  "compilerOptions": {
    "target": "ES2020",
    "useDefineForClassFields": true,
    "module": "ESNext",
    "lib": ["ES2020", "DOM", "DOM.Iterable"],
    "skipLibCheck": true,
    "moduleResolution": "bundler",
    "allowImportingTsExtensions": true,
    "resolveJsonModule": true,
    "isolatedModules": true,
    "noEmit": true,
    "jsx": "preserve",
    "strict": true,
    "noUnusedLocals": true,
    "noUnusedParameters": true,
    "noFallthroughCasesInSwitch": true,
    "baseUrl": ".",
    "paths": {
      "@/*": ["frontend/src/*"]
    }
  },
  "include": ["frontend/src/**/*.ts", "frontend/src/**/*.tsx", "frontend/src/**/*.vue"],
  "references": [{ "path": "./tsconfig.node.json" }]
}
```

### 3.4 tsconfig.node.json

```json
{
  "compilerOptions": {
    "composite": true,
    "skipLibCheck": true,
    "module": "ESNext",
    "moduleResolution": "bundler",
    "allowSyntheticDefaultImports": true
  },
  "include": ["vite.config.ts"]
}
```

### 3.5 index.html

```html
<!doctype html>
<html lang="zh-CN">
  <head>
    <meta charset="UTF-8" />
    <link rel="icon" type="image/svg+xml" href="/vite.svg" />
    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
    <title>牛马信仰</title>
  </head>
  <body>
    <div id="app"></div>
    <script type="module" src="/frontend/src/main.ts"></script>
  </body>
</html>
```

### 3.6 src-tauri/Cargo.toml

```toml
[package]
name = "niuma-faith"
version = "0.1.0"
edition = "2021"

[lib]
name = "niuma_faith_lib"
crate-type = ["staticlib", "cdylib", "rlib"]

[[bin]]
name = "niuma-faith"
path = "src/main.rs"
required-features = ["desktop"]

[build-dependencies]
tauri-build = { version = "2", features = [] }

[dependencies]
tauri = { version = "2", features = ["tray-icon", "test"], optional = true }
tauri-plugin-shell = { version = "2", optional = true }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
rusqlite = { version = "0.31", features = ["bundled"] }
chrono = { version = "0.4", features = ["serde"] }
thiserror = "1"
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
dirs = "5"

[dev-dependencies]
tempfile = "3"

[features]
default = ["desktop", "custom-protocol"]
desktop = ["dep:tauri", "dep:tauri-plugin-shell"]
custom-protocol = ["desktop", "tauri/custom-protocol"]
```

### 3.7 src-tauri/tauri.conf.json

```json
{
  "build": {
    "beforeDevCommand": "npm run dev",
    "beforeBuildCommand": "npm run build",
    "devUrl": "http://localhost:5173",
    "frontendDist": "../frontend/dist"
  },
  "productName": "牛马信仰",
  "version": "0.1.0",
  "identifier": "com.niuma.faith",
  "bundle": {
    "active": true,
    "targets": ["nsis"],
    "icon": [
      "icons/32x32.png",
      "icons/128x128.png",
      "icons/128x128@2x.png",
      "icons/icon.icns",
      "icons/icon.ico"
    ]
  }
}
```

---

## 4. 构建步骤

### 4.1 安装前端依赖

```bash
cd niuma-faith
npm install
```

### 4.2 前端开发模式

```bash
npm run dev
# 浏览器访问 http://localhost:5173
# 此时使用 localStorage Mock（完整业务逻辑）
```

### 4.3 类型检查

```bash
npx vue-tsc --noEmit
```

### 4.4 前端生产构建

```bash
npm run build
# 输出到 frontend/dist/
```

### 4.5 运行 Rust 测试

```bash
cd src-tauri
cargo test
# 期望: 141 passed
```

### 4.6 完整 Tauri 开发模式

```bash
npm run tauri dev
# 或
cargo tauri dev
# 启动完整的桌面应用（前端 + 后端 + WebView）
```

### 4.7 生产构建

```bash
npm run tauri build
# 输出到 src-tauri/target/release/
# Windows 安装包: src-tauri/target/release/bundle/nsis/*.exe
```

---

## 5. 数据还原

若需还原用户数据（从备份）：

```
数据目录: %LOCALAPPDATA%\牛马信仰\niuma_faith.db
        (即 C:\Users\{用户名}\AppData\Local\牛马信仰\niuma_faith.db)

开发环境回退: niuma-faith.exe 所在目录\niuma_faith.db
```

直接复制 `.db` 文件到上述路径即可恢复数据。

---

## 6. 还原检查清单

从零开始还原时，按此清单验证：

### 6.1 编译检查

- [ ] `npm install` 成功
- [ ] `npx vue-tsc --noEmit` 0 错误
- [ ] `npm run build` 成功
- [ ] `cd src-tauri && cargo test` 141 passed
- [ ] `npm run tauri dev` 能启动应用

### 6.2 功能检查

- [ ] 仪表盘页面加载，显示日历 + 任务列表
- [ ] 创建任务成功，出现在列表中
- [ ] 开始/暂停/继续任务，计时器正常
- [ ] 完成任务，显示信仰奖励
- [ ] 打卡功能，信仰计算正确
- [ ] 看板页面加载，默认四列
- [ ] 看板拖拽卡片正常
- [ ] 创建每日重复任务，第二天出现虚拟实例
- [ ] 悬浮窗可打开/关闭
- [ ] 系统托盘菜单正常
- [ ] 进程绑定（Windows）自动启停

### 6.3 数据检查

- [ ] 数据库文件在正确位置生成
- [ ] 5 张表结构正确
- [ ] 升级时 current_level 自动更新
- [ ] 护甲系统正确赋值
