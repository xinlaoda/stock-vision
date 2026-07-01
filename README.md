# Stock Vision

跨平台桌面股票分析工具 — A股 + 美股，基本面分析 + 技术分析 + 实时行情。

## 功能一览

### ✅ Phase 1：A股基础功能（全部完成）

| 模块 | 功能 |
|------|------|
| **股票搜索** | 东方财富 API 搜索，支持代码/名称/拼音 |
| **K 线图** | 日/周/月/年K + 蜡烛图 + 成交量柱 + MA5/10/20/60 均线 |
| **分时图** | 独立折线图 + 成交量柱 + VWAP 均价线 + 十字光标 |
| **分时数据** | 5/15/30/60分钟 + 1分钟分时数据 |
| **时间范围** | 1月/3月/6月/1年/2年/5年/年初至今/全部 |
| **交互** | 鼠标滚轮缩放 / 拖拽平移 / 十字光标 Tooltip |
| **首页** | 上证/深证/创业板/科创50 指数 + 走势图 + 自适应网格 |
| **基本面** | EPS/ROE/营收/利润/负债/现金流 + 健康评分 |
| **自选股** | 搜索添加 + 删除 + 持久化（重启保留）+ 快捷导航 |
| **浏览历史** | 点击量自动排序 + 持久化 |
| **后台同步** | 智能缓存判断 + 自动加载股票数据 |
| **缓存** | SQLite 本地持久化，已缓存数据不重复请求 |
| **设置** | 数据源信息 / 缓存状态 / 主题切换 / 关于 |
| **导出** | K 线数据导出 CSV |

### ✅ Phase 2：技术分析（全部完成）

| 模块 | 功能 |
|------|------|
| **MA 均线** | MA5/10/20/60 叠加 + 参数可调 |
| **MACD** | DIF/DEA 线 + 柱状图 + 参数可调 |
| **KDJ** | K/D/J 三线 + 参数可调 |
| **RSI** | 超买超卖线 (70/30) + 参数可调 |
| **BOLL** | 布林带上中下三轨 + 参数可调 |
| **技术面板** | 指标开关切换 + 当前值显示 + 参数编辑 |
| **动态副图** | MACD/KDJ/RSI 根据选择自动切换 |
| **画线工具** | 水平线 / 趋势线(两点) / 射线 / 平行通道 |
| **实时行情** | 每 5 秒自动轮询更新最新价格 |

### 📅 Phase 3：量化回测（规划中）

- 回测引擎 + 策略框架
- 金叉死叉 / 均线突破策略
- 收益率曲线 / 胜率 / 最大回撤

## 技术栈

| 层 | 技术 | 说明 |
|----|------|------|
| **语言** | Rust | 性能、内存安全、跨平台编译 |
| **GUI** | [Iced](https://github.com/iced-rs/iced) 0.14 (Elm 架构) | 原生跨平台，声明式 UI |
| **图表** | Iced Canvas (原生) | 高性能自定义绘制 |
| **网络** | reqwest | HTTP 数据源 |
| **存储** | SQLite (rusqlite) | 数据缓存 + 持久化 |
| **运行时** | Tokio | 异步数据加载 |
| **数据源** | 腾讯/东方财富(A股) + Yahoo/Finnhub(美股) 多源 Fallback | A股+美股实时+历史 |

## 架构

```
stock-vision/
├── src/                       # 主应用
│   ├── main.rs               # iced entry point
│   ├── app.rs                # Elm 架构: Model / Update / View
│   ├── state/mod.rs          # 应用状态 (AppState / KlinePeriod / TimeRange)
│   ├── ui/
│   │   ├── style.rs          # 配色系统 + 亮色/深色主题
│   │   ├── charts/
│   │   │   ├── candlestick_chart.rs  # K 线图 (三区布局: K线+成交量+副图)
│   │   │   └── intraday_chart.rs     # 分时走势图 (折线+成交量+VWAP)
│   │   └── panels/
│   │       ├── home.rs       # 首页市场概况
│   │       ├── watchlist.rs  # 自选股
│   │       ├── chart.rs      # 行情走势 (K线/分时 + 技术指标)
│   │       ├── fundamental.rs # 基本面分析
│   │       ├── technical.rs  # 技术分析 (指标开关 + 参数配置)
│   │       └── settings.rs   # 设置 (主题/数据源/缓存)
│   └── services/
│       ├── data_service.rs   # 数据服务 (缓存 + Fallback 多源)
│       ├── analysis_service.rs # 分析服务
│       └── indicator_service.rs # 指标计算服务
├── crates/
│   ├── data-model/           # 领域模型 (Stock/DailyBar/FinancialReport/Exchange)
│   ├── data-source/          # 数据源 (Tencent/EastMoney/Yahoo/Finnhub/Mock + FallbackSource)
│   ├── indicator-core/       # 技术指标计算 (SMA/EMA/MACD/RSI/KDJ/BOLL)
│   ├── analysis-core/        # 基本面分析/财务健康评分
│   ├── chart-engine/         # 图表引擎 (plotters 封装，备用)
│   └── storage/              # SQLite 持久化层
```

## 快速开始

```bash
# 前置条件: Rust 1.75+
git clone https://github.com/xinlaoda/stock-vision.git
cd stock-vision

# 运行开发版本（A股数据无需配置，开箱即用）
cargo run

# 启用美股 Finnhub 数据（可选，注册 https://finnhub.io/register）
export FINNHUB_API_KEY="your_api_key_here"
cargo run

# 发布构建
cargo build --release
```

### Windows 注意事项

```bash
# Windows 端
git pull

# 设置 Finnhub key（可选）
$env:FINNHUB_API_KEY="your_api_key_here"

cargo run
```

默认字体设置为 Microsoft YaHei，解决中文显示问题。

## 数据源

### A股
| 功能 | 主源 | 备源 | 
|------|------|------|
| 股票搜索 | 东方财富 `searchadapter.eastmoney.com` | - |
| 日K线 | 腾讯财经 `web.ifzq.gtimg.cn` | - |
| 分时数据 | 腾讯 `mkline` 接口 | 东方财富 `push2his` |
| 基本面 | 东方财富 `datacenter.eastmoney.com` | - |
| 实时行情 | 腾讯财经 (每5秒轮询) | - |

### 美股
| 功能 | 主源 | 备源 | 
|------|------|------|
| 股票搜索 | EastMoney → Yahoo → Finnhub（三级fallback） | - |
| 日K/周K/月K线 | Finnhub（有key时） | Yahoo Finance（无条件fallback） |
| 基本面 | Finnhub（需API key） | - |
| 估值(PE/PB) | Finnhub（需API key） | - |

> **Finnhub**: 免费注册 https://finnhub.io/register，设置环境变量 `FINNHUB_API_KEY` 即可启用更全面的美股数据（基本面/估值/实时）。
> **Yahoo Finance**: 免费无需注册，数据可靠但非官方API，延迟约15分钟。

所有数据源采用 **Fallback 策略**，主源失败自动切换到备源。

## 技术特色

- **多源 Fallback**: 4个数据源冗余，自动切换（Tencent → EastMoney → Finnhub → Yahoo）
- **智能缓存**: SQLite本地缓存，已缓存数据不重复请求网络
- **A股+美股**: 自动识别交易所，选择对应数据源
- **响应式布局**: 首页指数卡片根据窗口宽度自适应 1/2 列
- **跨平台**: 一次编写，Windows/macOS/Linux 均可编译运行

## 许可

MIT
