# Stock Vision

跨平台桌面股票分析工具 — 从基本面分析起步，逐步扩展至技术分析和量化回测。

## 项目目标

构建一款 **面向普通投资者的 A 股桌面分析软件**，对标通达信/同花顺/东方财富的核心功能，
同时保持现代化跨平台体验和开源自由。

### 路线图

| 阶段 | 功能 | 状态 |
|------|------|------|
| **Phase 1** | 基本面分析 + 行情图表 + 自选股管理 | 🚧 进行中 |
| **Phase 2** | 技术指标 (MA/MACD/RSI/BOLL) + 技术图表 | 📅 规划 |
| **Phase 3** | 量化回测引擎 + 策略编辑器 | 📅 规划 |

## 技术栈

| 层 | 技术 | 说明 |
|----|------|------|
| **语言** | Rust | 性能、内存安全、跨平台编译 |
| **GUI 框架** | [Iced](https://github.com/iced-rs/iced) (Elm 架构) | 原生跨平台，声明式 UI |
| **图表引擎** | [Plotters](https://github.com/plotters-rs/plotters) | 金融 K 线/指标渲染 |
| **网络请求** | reqwest + serde | HTTP API 调用 |
| **本地存储** | SQLite (rusqlite) | 数据缓存 + 配置 |
| **数据源** | 东方财富 API / Tushare | A 股实时 + 历史数据 |
| **运行时** | Tokio | 异步数据加载 |

### 架构

```
stock-vision/
├── src/                    # 主应用 (Iced Sandbox)
│   ├── app.rs             # Elm 架构: Model / Update / View
│   ├── state/             # 应用状态管理
│   ├── ui/
│   │   ├── panels/        # 功能面板 (自选/行情/基本面/技术/设置)
│   │   ├── widgets/       # 可复用 UI 组件
│   │   └── style.rs       # 主题和样式
│   └── services/          # 业务逻辑编排
├── crates/
│   ├── data-model/        # 核心领域模型 (Stock/Bars/Reports/Indicators)
│   ├── data-source/       # 数据源抽象层 (EastMoney/Mock)
│   ├── chart-engine/      # 图表渲染引擎 (plotters 封装)
│   ├── analysis-core/     # 基本面分析/财务健康评分
│   ├── indicator-core/    # 技术指标计算 (SMA/EMA/MACD/RSI/BOLL)
│   └── storage/           # SQLite 持久化层
└── docs/                  # 设计文档
```

## 快速开始

```bash
# 克隆
git clone <repo-url>

# 运行
cargo run

# 开发模式
cargo watch -x run
```

## Phase 1 功能详情

### 基本面分析
- 公司财务数据展示 (营收/利润/资产/负债/现金流)
- 财务健康评分系统 (ROE/负债率/利润率/估值多维评分)
- 估值指标 (PE/PB/PS/PCF/股息率)

### 行情图表
- K 线图 (日线/复权支持)
- 价格走势概览

### 自选股
- 搜索添加到自选
- 自选股列表管理

## 数据源

### 东方财富 API (待实现完整解析)
- 搜索: `searchadapter.eastmoney.com`
- K 线: `push2his.eastmoney.com`
- 财务: EastMoney finance API

### Mock 数据源 (开发调试用)
- 内置模拟数生成器

## 支持市场

- **Phase 1**: A 股 (深交所/上交所)
- **未来**: 美股、港股

## 许可

MIT

## API 对接状态

### ✅ 已对接并验证（Phase 1）

| 功能 | 数据源 | API | 状态 |
|------|--------|-----|------|
| **股票搜索** | 东方财富 | `searchadapter.eastmoney.com` | ✅ 支持中文/代码搜索，过滤A股 |
| **日K线** | 腾讯财经 | `web.ifzq.gtimg.cn` | ✅ 前复权，2000条上限 |
| **数据缓存** | SQLite | `rusqlite` | ✅ 本地持久化 |

### 📅 待对接

| 功能 | 数据源 | 优先级 |
|------|--------|--------|
| 财务报告 | 新浪/东方财富 | P1 - Phase 1 关键 |
| 实时行情 | 腾讯/新浪 | P2 |
| 估值指标 | 东方财富 | P2 |
| 分钟K线 | 腾讯 | P3 |

### 代码验证

```bash
# 运行 API 测试
cargo test test_search -- --nocapture
cargo test test_tencent_kline -- --nocapture
```
