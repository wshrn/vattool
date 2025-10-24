# 统一风格与离线认证开发文档

本开发文档汇总了项目中用于实现统一界面风格、深浅主题切换与离线密钥认证的全部通用代码。通过遵循本文档，其他开发人员无需了解原始工程背景，即可实现相同的视觉样式、交互体验以及离线密钥校验机制。

## 目录

1. [前端基础与样式体系](#前端基础与样式体系)
2. [主题状态管理与切换组件](#主题状态管理与切换组件)
3. [窗口标题栏与操作控件](#窗口标题栏与操作控件)
4. [主要页面结构](#主要页面结构)
5. [离线密钥服务（前端）](#离线密钥服务前端)
6. [应用入口初始化流程](#应用入口初始化流程)
7. [后端配置与主题持久化](#后端配置与主题持久化)
8. [离线密钥认证核心逻辑（后端）](#离线密钥认证核心逻辑后端)
9. [设备指纹与存储辅助模块](#设备指纹与存储辅助模块)

---

## 前端基础与样式体系

创建 `src/styles/theme.css`，包含完整的浅色/深色主题视觉定义、组件皮肤以及自定义标题栏样式。该文件基于 TailwindCSS 层叠结构构建，同时对夜间模式进行了细腻的发光与玻璃拟态处理。

```css
@tailwind base;
@tailwind components;
@tailwind utilities;

@layer base {
  :root {
    --app-surface-bg: #f8fafc;
    --app-surface-border: rgba(148, 163, 184, 0.25);
    --dialog-surface: rgba(255, 255, 255, 0.95);
    --dialog-border: rgba(148, 163, 184, 0.24);
    color-scheme: light;
  }

  html.dark {
    --app-surface-bg: #1f2937;
    --app-surface-border: #374151;
    --dialog-surface: rgba(15, 23, 42, 0.96);
    --dialog-border: rgba(59, 130, 246, 0.25);
    color-scheme: dark;
  }

  html,
  body {
    height: 100%;
  }

  body {
    @apply antialiased;
    margin: 0;
    overflow: hidden;
    background: transparent;
  }

  #app {
    height: 100%;
  }
}

@layer components {
  .app-shell {
    @apply relative min-h-0 transition-colors duration-500 ease-apple text-slate-900;
    background: radial-gradient(120% 120% at 100% 0%, rgba(59, 130, 246, 0.08) 0%, rgba(191, 219, 254, 0.12) 22%, transparent 70%),
      radial-gradient(120% 120% at 0% 100%, rgba(52, 211, 153, 0.14) 0%, rgba(14, 165, 233, 0.12) 32%, transparent 70%),
      linear-gradient(135deg, #f8fafc, #e0f2fe);
    color: #0f172a;
    overflow: hidden;
  }

  .dark .app-shell {
    color: #e2e8f0;
    background: radial-gradient(120% 120% at 100% 0%, rgba(56, 189, 248, 0.12) 0%, rgba(59, 130, 246, 0.08) 24%, transparent 70%),
      radial-gradient(120% 120% at 0% 100%, rgba(14, 165, 233, 0.08) 0%, rgba(16, 185, 129, 0.05) 32%, transparent 70%),
      linear-gradient(135deg, rgba(15, 23, 42, 0.96), rgba(2, 6, 23, 0.98));
  }

  .app-shell__glow {
    @apply pointer-events-none absolute rounded-full blur-3xl opacity-60;
  }

  .app-shell__glow--primary {
    inset: auto -18rem -18rem auto;
    width: 28rem;
    height: 28rem;
    background: radial-gradient(circle at center, rgba(59, 130, 246, 0.3), rgba(59, 130, 246, 0));
  }

  .app-shell__glow--secondary {
    inset: -18rem auto auto -16rem;
    width: 32rem;
    height: 32rem;
    background: radial-gradient(circle at center, rgba(16, 185, 129, 0.25), rgba(16, 185, 129, 0));
  }

  .dark .app-shell__glow--primary {
    background: radial-gradient(circle at center, rgba(56, 189, 248, 0.6), rgba(37, 99, 235, 0));
  }

  .dark .app-shell__glow--secondary {
    background: radial-gradient(circle at center, rgba(14, 165, 233, 0.55), rgba(16, 185, 129, 0));
  }

  .app-panel {
    @apply relative w-full px-10 py-12 text-slate-900;
    display: grid;
    gap: 2.5rem;
    overflow: hidden;
    border-radius: 28px;
  }

  .app-panel::before,
  .app-panel::after {
    content: '';
    position: absolute;
    inset: 1px;
    border-radius: 26px;
    pointer-events: none;
  }

  .app-panel::before {
    background: rgba(255, 255, 255, 0.9);
    border: 1px solid rgba(148, 163, 184, 0.25);
    box-shadow: 0 30px 60px rgba(148, 163, 184, 0.25);
    backdrop-filter: blur(18px);
  }

  .app-panel::after {
    background: radial-gradient(120% 120% at 100% 0%, rgba(59, 130, 246, 0.12), transparent 60%);
    mix-blend-mode: normal;
  }

  .dark .app-panel {
    @apply text-gray-100;
  }

  .dark .app-panel::before {
    background: rgba(15, 23, 42, 0.82);
    border: 1px solid rgba(59, 130, 246, 0.2);
    box-shadow: 0 40px 80px rgba(15, 23, 42, 0.45);
    backdrop-filter: blur(14px);
  }

  .dark .app-panel::after {
    background: radial-gradient(120% 120% at 100% 0%, rgba(37, 99, 235, 0.16), transparent 60%);
    mix-blend-mode: screen;
  }

  .panel-header,
  .panel-section,
  .panel-footer,
  .dialog-backdrop {
    position: relative;
    z-index: 1;
  }

  .panel-section--mirrors {
    z-index: 3;
  }

  .panel-header {
    @apply space-y-4;
  }

  .panel-chip {
    @apply inline-flex items-center gap-2 rounded-full px-3 py-1 text-[11px] font-semibold;
    color: #0369a1;
    background: rgba(125, 211, 252, 0.18);
    border: 1px solid rgba(56, 189, 248, 0.35);
    box-shadow: inset 0 0 0 1px rgba(14, 165, 233, 0.25);
  }

  .dark .panel-chip {
    @apply text-sky-200;
    background: rgba(56, 189, 248, 0.15);
    border: 1px solid rgba(56, 189, 248, 0.35);
    box-shadow: inset 0 0 0 1px rgba(14, 165, 233, 0.35);
  }

  .panel-title {
    @apply text-3xl font-semibold tracking-tight;
    color: #0f172a;
  }

  .dark .panel-title {
    @apply text-slate-100;
  }

  .panel-subtitle {
    @apply mt-2 max-w-xl text-sm leading-relaxed;
    color: rgba(71, 85, 105, 0.85);
  }

  .dark .panel-subtitle {
    @apply text-slate-300/80;
  }

  .panel-section {
    @apply space-y-5;
  }

  .panel-section__title-wrap {
    @apply flex flex-wrap items-end justify-between gap-2;
  }

  .panel-section__title {
    @apply text-sm font-semibold uppercase tracking-[0.3em];
    color: rgba(30, 64, 175, 0.8);
  }

  .dark .panel-section__title {
    @apply text-slate-200/90;
  }

  .panel-section__hint {
    @apply text-xs;
    color: rgba(7, 89, 133, 0.7);
  }

  .dark .panel-section__hint {
    @apply text-sky-200/70;
  }

  .status-grid {
    @apply grid gap-3 sm:grid-cols-2;
  }

  .status-card {
    @apply relative flex items-start gap-4 overflow-hidden rounded-2xl border px-5 py-4 transition-colors duration-300;
    border-color: rgba(148, 163, 184, 0.2);
    background: linear-gradient(135deg, rgba(241, 245, 249, 0.95), rgba(255, 255, 255, 0.96));
    box-shadow: inset 0 0 0 1px rgba(148, 163, 184, 0.08), 0 18px 28px rgba(148, 163, 184, 0.18);
  }

  .status-card::after {
    content: '';
    position: absolute;
    inset: 0;
    background: linear-gradient(135deg, rgba(59, 130, 246, 0.12), transparent 55%);
    opacity: 0;
    transition: opacity 0.3s ease;
  }

  .status-card:hover::after {
    opacity: 1;
  }

  .status-card--ok {
    border-color: rgba(16, 185, 129, 0.35);
    background: linear-gradient(135deg, rgba(16, 185, 129, 0.1), rgba(255, 255, 255, 0.96));
  }

  .status-card--error {
    border-color: rgba(248, 113, 113, 0.4);
    background: linear-gradient(135deg, rgba(248, 113, 113, 0.12), rgba(255, 255, 255, 0.98));
  }

  .dark .status-card {
    @apply border border-white/5 bg-white/5;
    box-shadow: inset 0 0 0 1px rgba(148, 163, 184, 0.08);
  }

  .dark .status-card::after {
    background: linear-gradient(135deg, rgba(56, 189, 248, 0.15), transparent 45%);
  }

  .dark .status-card--ok {
    border-color: rgba(16, 185, 129, 0.32);
    background: linear-gradient(135deg, rgba(16, 185, 129, 0.18), rgba(15, 23, 42, 0.9));
  }

  .dark .status-card--error {
    border-color: rgba(248, 113, 113, 0.4);
    background: linear-gradient(135deg, rgba(248, 113, 113, 0.2), rgba(15, 23, 42, 0.92));
  }

  .status-card__icon {
    @apply relative flex h-10 w-10 items-center justify-center rounded-xl;
    background: linear-gradient(135deg, rgba(14, 165, 233, 0.25), rgba(59, 130, 246, 0.08));
    box-shadow: inset 0 0 12px rgba(56, 189, 248, 0.25);
  }

  .dark .status-card__icon {
    @apply bg-slate-900/80;
    box-shadow: inset 0 0 12px rgba(56, 189, 248, 0.4);
  }

  .status-card__beam {
    @apply absolute h-8 w-px bg-gradient-to-b from-transparent via-sky-400 to-transparent;
  }

  .dark .status-card__beam {
    @apply bg-gradient-to-b from-transparent via-sky-300 to-transparent;
  }

  .status-card__dot {
    @apply relative h-2 w-2 rounded-full;
    background: #0284c7;
    box-shadow: 0 0 12px rgba(14, 165, 233, 0.6);
  }

  .dark .status-card__dot {
    @apply bg-sky-200;
    box-shadow: 0 0 12px rgba(94, 234, 212, 0.8);
  }

  .status-card__label {
    @apply text-sm font-semibold;
    color: #0f172a;
  }

  .dark .status-card__label {
    @apply text-slate-100;
  }

  .status-card__message {
    @apply mt-1 text-xs;
    color: #475569;
    white-space: pre-line;
  }

  .dark .status-card__message {
    @apply text-slate-300;
  }

  .status-card__detail {
    @apply mt-1 text-[11px];
    color: #64748b;
  }

  .dark .status-card__detail {
    @apply text-slate-400;
  }

  .mirror-select {
    @apply space-y-3;
  }
  .panel-header {
    @apply space-y-4;
  }

  .panel-chip {
    @apply inline-flex items-center gap-2 rounded-full px-3 py-1 text-[11px] font-semibold;
    color: #0369a1;
    background: rgba(125, 211, 252, 0.18);
    border: 1px solid rgba(56, 189, 248, 0.35);
    box-shadow: inset 0 0 0 1px rgba(14, 165, 233, 0.25);
  }

  .dark .panel-chip {
    @apply text-sky-200;
    background: rgba(56, 189, 248, 0.15);
    border: 1px solid rgba(56, 189, 248, 0.35);
    box-shadow: inset 0 0 0 1px rgba(14, 165, 233, 0.35);
  }

  .panel-title {
    @apply text-3xl font-semibold tracking-tight;
    color: #0f172a;
  }

  .dark .panel-title {
    @apply text-slate-100;
  }

  .panel-subtitle {
    @apply mt-2 max-w-xl text-sm leading-relaxed;
    color: rgba(71, 85, 105, 0.85);
  }

  .dark .panel-subtitle {
    @apply text-slate-300/80;
  }

  .panel-section {
    @apply space-y-5;
  }

  .panel-section__title-wrap {
    @apply flex flex-wrap items-end justify-between gap-2;
  }

  .panel-section__title {
    @apply text-sm font-semibold uppercase tracking-[0.3em];
    color: rgba(30, 64, 175, 0.8);
  }

  .dark .panel-section__title {
    @apply text-slate-200/90;
  }

  .panel-section__hint {
    @apply text-xs;
    color: rgba(7, 89, 133, 0.7);
  }

  .dark .panel-section__hint {
    @apply text-sky-200/70;
  }

  .status-grid {
    @apply grid gap-3 sm:grid-cols-2;
  }

  .status-card {
    @apply relative flex items-start gap-4 overflow-hidden rounded-2xl border px-5 py-4 transition-colors duration-300;
    border-color: rgba(148, 163, 184, 0.2);
    background: linear-gradient(135deg, rgba(241, 245, 249, 0.95), rgba(255, 255, 255, 0.96));
    box-shadow: inset 0 0 0 1px rgba(148, 163, 184, 0.08), 0 18px 28px rgba(148, 163, 184, 0.18);
  }

  .status-card::after {
    content: '';
    position: absolute;
    inset: 0;
    background: linear-gradient(135deg, rgba(59, 130, 246, 0.12), transparent 55%);
    opacity: 0;
    transition: opacity 0.3s ease;
  }

  .status-card:hover::after {
    opacity: 1;
  }

  .status-card--ok {
    border-color: rgba(16, 185, 129, 0.35);
    background: linear-gradient(135deg, rgba(16, 185, 129, 0.1), rgba(255, 255, 255, 0.96));
  }

  .status-card--error {
    border-color: rgba(248, 113, 113, 0.4);
    background: linear-gradient(135deg, rgba(248, 113, 113, 0.12), rgba(255, 255, 255, 0.98));
  }

  .dark .status-card {
    @apply border border-white/5 bg-white/5;
    box-shadow: inset 0 0 0 1px rgba(148, 163, 184, 0.08);
  }

  .dark .status-card::after {
    background: linear-gradient(135deg, rgba(56, 189, 248, 0.15), transparent 45%);
  }

  .dark .status-card--ok {
    border-color: rgba(16, 185, 129, 0.32);
    background: linear-gradient(135deg, rgba(16, 185, 129, 0.18), rgba(15, 23, 42, 0.9));
  }

  .dark .status-card--error {
    border-color: rgba(248, 113, 113, 0.4);
    background: linear-gradient(135deg, rgba(248, 113, 113, 0.2), rgba(15, 23, 42, 0.92));
  }

  .status-card__icon {
    @apply relative flex h-10 w-10 items-center justify-center rounded-xl;
    background: linear-gradient(135deg, rgba(14, 165, 233, 0.25), rgba(59, 130, 246, 0.08));
    box-shadow: inset 0 0 12px rgba(56, 189, 248, 0.25);
  }

  .dark .status-card__icon {
    @apply bg-slate-900/80;
    box-shadow: inset 0 0 12px rgba(56, 189, 248, 0.4);
  }

  .status-card__beam {
    @apply absolute h-8 w-px bg-gradient-to-b from-transparent via-sky-400 to-transparent;
  }

  .dark .status-card__beam {
    @apply bg-gradient-to-b from-transparent via-sky-300 to-transparent;
  }

  .status-card__dot {
    @apply relative h-2 w-2 rounded-full;
    background: #0284c7;
    box-shadow: 0 0 12px rgba(14, 165, 233, 0.6);
  }

  .dark .status-card__dot {
    @apply bg-sky-200;
    box-shadow: 0 0 12px rgba(94, 234, 212, 0.8);
  }

  .status-card__label {
    @apply text-sm font-semibold;
    color: #0f172a;
  }

  .dark .status-card__label {
    @apply text-slate-100;
  }

  .status-card__message {
    @apply mt-1 text-xs;
    color: #475569;
    white-space: pre-line;
  }

  .dark .status-card__message {
    @apply text-slate-300;
  }

  .status-card__detail {
    @apply mt-1 text-[11px];
    color: #64748b;
  }

  .dark .status-card__detail {
    @apply text-slate-400;
  }

  .mirror-select {
    @apply space-y-3;
  }
  .mirror-select__label {
    @apply text-xs font-medium uppercase tracking-[0.22em];
    color: rgba(30, 64, 175, 0.7);
  }

  .dark .mirror-select__label {
    @apply text-slate-200/80;
  }

  .mirror-select__input {
    @apply w-full appearance-none rounded-2xl py-3.5 pl-14 pr-14 text-sm font-medium shadow-inner focus:outline-none focus:ring-2 focus:ring-sky-300/40 focus:ring-offset-2 focus:ring-offset-white;
    @apply flex items-center justify-between text-left gap-3;
    background: linear-gradient(135deg, rgba(59, 130, 246, 0.08), rgba(14, 165, 233, 0.05));
    color: #0f172a;
    border: 1px solid rgba(148, 163, 184, 0.35);
    backdrop-filter: blur(12px);
    transition: border-color 0.3s ease, box-shadow 0.3s ease, transform 0.2s ease;
    cursor: pointer;
  }

  .mirror-select__input:hover {
    box-shadow: 0 14px 28px rgba(148, 163, 184, 0.18);
    transform: translateY(-1px);
  }

  .mirror-select__input:focus {
    border-color: rgba(59, 130, 246, 0.55);
    box-shadow: 0 0 0 4px rgba(59, 130, 246, 0.15);
  }

  .mirror-select__input--disabled {
    @apply cursor-not-allowed opacity-60;
  }

  .mirror-select__input[data-open='true'] {
    border-color: rgba(59, 130, 246, 0.5);
    box-shadow: 0 18px 32px rgba(59, 130, 246, 0.12), 0 0 0 4px rgba(59, 130, 246, 0.12);
    transform: translateY(-1px);
  }

  .dark .mirror-select__input {
    @apply text-slate-100;
    background: linear-gradient(135deg, rgba(30, 64, 175, 0.45), rgba(15, 23, 42, 0.75));
    border: 1px solid rgba(148, 163, 184, 0.25);
  }

  .dark .mirror-select__input:focus {
    border-color: rgba(56, 189, 248, 0.55);
    box-shadow: 0 0 0 4px rgba(56, 189, 248, 0.15);
  }

  .dark .mirror-select__input[data-open='true'] {
    border-color: rgba(125, 211, 252, 0.5);
    box-shadow: 0 20px 36px rgba(15, 23, 42, 0.45), 0 0 0 4px rgba(125, 211, 252, 0.12);
  }

  .mirror-select__control {
    @apply relative flex items-center;
  }

  .mirror-select__icon {
    @apply pointer-events-none absolute left-4 top-1/2 flex h-9 w-9 -translate-y-1/2 items-center justify-center rounded-xl text-sky-600/90;
    background: linear-gradient(145deg, rgba(125, 211, 252, 0.2), rgba(59, 130, 246, 0.08));
    box-shadow: inset 0 0 0 1px rgba(59, 130, 246, 0.18), 0 10px 18px rgba(148, 163, 184, 0.18);
  }

  .mirror-select__icon svg {
    width: 1.25rem;
    height: 1.25rem;
  }

  .dark .mirror-select__icon {
    @apply text-sky-200/90;
    background: linear-gradient(145deg, rgba(37, 99, 235, 0.25), rgba(14, 165, 233, 0.12));
    box-shadow: inset 0 0 0 1px rgba(59, 130, 246, 0.3), 0 12px 24px rgba(2, 6, 23, 0.45);
  }

  .mirror-select__chevron {
    @apply pointer-events-none absolute inset-y-0 right-5 flex items-center text-sky-700/70;
  }

  .mirror-select__chevron svg {
    width: 1.1rem;
    height: 1.1rem;
  }

  .dark .mirror-select__chevron {
    @apply text-sky-200/80;
  }

  .mirror-select__value {
    @apply flex min-w-0 flex-col;
  }

  .mirror-select__value-label {
    @apply truncate text-sm font-semibold;
    color: rgba(15, 23, 42, 0.9);
    letter-spacing: 0.01em;
  }

  .mirror-select__value-url {
    @apply truncate text-xs font-medium;
    color: rgba(37, 99, 235, 0.8);
    letter-spacing: 0.02em;
  }

  .dark .mirror-select__value-label {
    @apply text-slate-100;
  }

  .dark .mirror-select__value-url {
    @apply text-sky-200;
    color: rgba(125, 211, 252, 0.75);
  }

  .mirror-select__value--empty {
    @apply justify-center;
  }

  .mirror-select__placeholder {
    @apply text-sm font-medium text-slate-400;
  }

  .mirror-select__shine {
    @apply pointer-events-none absolute inset-0 rounded-2xl;
    background: radial-gradient(circle at 15% 20%, rgba(255, 255, 255, 0.5), transparent 55%);
    opacity: 0.45;
    mix-blend-mode: screen;
  }

  .dark .mirror-select__shine {
    background: radial-gradient(circle at 18% 24%, rgba(148, 197, 253, 0.45), transparent 55%);
    opacity: 0.3;
    mix-blend-mode: lighten;
  }

  .mirror-select__border {
    @apply pointer-events-none absolute inset-0 rounded-2xl;
    border: 1px solid rgba(148, 163, 184, 0.15);
  }

  .dark .mirror-select__border {
    @apply border-sky-300/10;
  }

  .mirror-select__dropdown {
    @apply absolute left-0 right-0 z-20 mt-3 max-h-64 overflow-y-auto rounded-2xl border px-2 py-2 shadow-xl;
    background: linear-gradient(145deg, rgba(248, 250, 252, 0.96), rgba(241, 245, 249, 0.92));
    border-color: rgba(148, 163, 184, 0.25);
    box-shadow: 0 24px 45px rgba(15, 23, 42, 0.15), inset 0 0 0 1px rgba(255, 255, 255, 0.65);
    backdrop-filter: blur(18px);
  }

  .mirror-select__dropdown::-webkit-scrollbar {
    width: 6px;
  }

  .mirror-select__dropdown::-webkit-scrollbar-thumb {
    background: rgba(59, 130, 246, 0.45);
    border-radius: 9999px;
  }

  .mirror-select__dropdown::-webkit-scrollbar-track {
    background: transparent;
  }

  .dark .mirror-select__dropdown {
    background: linear-gradient(145deg, rgba(15, 23, 42, 0.92), rgba(30, 41, 59, 0.9));
    border-color: rgba(148, 163, 184, 0.2);
    box-shadow: 0 26px 50px rgba(2, 6, 23, 0.6), inset 0 0 0 1px rgba(148, 163, 184, 0.08);
  }

  .mirror-select__option {
    @apply flex flex-col gap-1 rounded-xl px-4 py-3 transition-all duration-200 ease-out;
    color: #0f172a;
    border: 1px solid transparent;
  }

  .mirror-select__option-label {
    @apply text-sm font-semibold;
    letter-spacing: 0.02em;
  }

  .mirror-select__option-url {
    @apply text-xs font-medium text-slate-500;
    letter-spacing: 0.02em;
  }

  .mirror-select__option--active {
    background: linear-gradient(135deg, rgba(59, 130, 246, 0.14), rgba(56, 189, 248, 0.12));
    border-color: rgba(59, 130, 246, 0.45);
    box-shadow: inset 0 0 0 1px rgba(59, 130, 246, 0.3);
    transform: translateX(3px);
  }

  .mirror-select__option--selected {
    border-color: rgba(37, 99, 235, 0.35);
    box-shadow: inset 0 0 0 1px rgba(37, 99, 235, 0.35);
  }

  .dark .mirror-select__option {
    @apply text-slate-100;
    border-color: transparent;
  }

  .dark .mirror-select__option-url {
    @apply text-sky-200/80;
  }

  .dark .mirror-select__option--active {
    background: linear-gradient(135deg, rgba(56, 189, 248, 0.18), rgba(30, 64, 175, 0.22));
    border-color: rgba(125, 211, 252, 0.45);
    box-shadow: inset 0 0 0 1px rgba(148, 197, 253, 0.35);
  }

  .dark .mirror-select__option--selected {
    border-color: rgba(148, 197, 253, 0.35);
    box-shadow: inset 0 0 0 1px rgba(148, 197, 253, 0.35);
  }

  .mirror-select__empty {
    @apply rounded-xl px-4 py-5 text-center text-sm font-medium text-slate-500;
  }

  .dark .mirror-select__empty {
    @apply text-slate-300;
  }

  .mirror-select__dropdown-transition-enter-active,
  .mirror-select__dropdown-transition-leave-active {
    transition: opacity 150ms ease, transform 150ms ease;
  }

  .mirror-select__dropdown-transition-enter-from,
  .mirror-select__dropdown-transition-leave-to {
    opacity: 0;
    transform: translateY(-8px) scale(0.98);
  }

  .mirror-select__note {
    @apply text-xs;
    color: rgba(71, 85, 105, 0.75);
  }

  .dark .mirror-select__note {
    @apply text-slate-400;
  }

  .panel-footer {
    @apply flex flex-col gap-4 rounded-2xl px-6 py-5 sm:flex-row sm:items-end sm:justify-between;
    background: rgba(255, 255, 255, 0.92);
    border: 1px solid rgba(148, 163, 184, 0.2);
    box-shadow: inset 0 0 0 1px rgba(255, 255, 255, 0.4), 0 12px 24px rgba(148, 163, 184, 0.22);
  }

  .dark .panel-footer {
    @apply border border-white/5 bg-white/5;
    box-shadow: inset 0 0 0 1px rgba(148, 163, 184, 0.06);
  }

  .panel-meta {
    @apply grid flex-1 gap-3 text-xs sm:grid-cols-3;
    color: #475569;
  }

  .dark .panel-meta {
    @apply text-slate-300;
  }

  .panel-meta dt {
    @apply font-semibold;
    color: #0f172a;
  }

  .dark .panel-meta dt {
    @apply text-slate-100/90;
  }

  .panel-meta dd {
    @apply mt-1 break-all text-[11px];
    color: #64748b;
  }

  .dark .panel-meta dd {
    @apply text-slate-400;
  }
  .btn-apple {
    @apply inline-flex items-center justify-center gap-2 rounded-full px-6 py-3 text-sm font-semibold uppercase tracking-[0.32em] text-slate-900 transition-all duration-300 ease-apple focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-offset-white disabled:cursor-not-allowed disabled:opacity-60;
    letter-spacing: 0.32em;
  }

  .dark .btn-apple {
    @apply focus:ring-offset-slate-900;
  }

  .btn-primary {
    @apply btn-apple bg-gradient-to-r from-sky-400 via-blue-500 to-violet-500 text-slate-900;
    box-shadow: 0 20px 45px rgba(56, 189, 248, 0.35), inset 0 1px 0 rgba(255, 255, 255, 0.35);
  }

  .btn-primary:hover {
    box-shadow: 0 24px 60px rgba(56, 189, 248, 0.45), inset 0 1px 0 rgba(255, 255, 255, 0.45);
    filter: saturate(1.15);
  }

  .panel-action {
    @apply whitespace-nowrap;
  }

  .panel-action__loading {
    @apply flex items-center gap-3;
  }

  .panel-action__spinner {
    @apply h-4 w-4 animate-spin;
  }

  .dialog-enter-active,
  .dialog-leave-active {
    transition: opacity 0.25s ease;
  }

  .dialog-enter-from,
  .dialog-leave-to {
    opacity: 0;
  }

  .dialog-backdrop {
    @apply fixed inset-0 z-50 flex items-center justify-center px-6;
    background: rgba(15, 23, 42, 0.2);
    backdrop-filter: blur(10px);
  }

  .dialog-panel {
    @apply w-full max-w-sm rounded-3xl border px-6 py-6 text-sm shadow-2xl;
    background: var(--dialog-surface);
    border-color: var(--dialog-border);
    color: #0f172a;
    box-shadow: 0 24px 60px rgba(148, 163, 184, 0.35);
  }

  .dark .dialog-panel {
    color: #e2e8f0;
    box-shadow: 0 24px 60px rgba(15, 23, 42, 0.55);
  }

  .dialog-panel.success {
    border-color: rgba(16, 185, 129, 0.35);
  }

  .dialog-panel.error {
    border-color: rgba(248, 113, 113, 0.4);
  }

  .dialog-panel.warning {
    border-color: rgba(250, 204, 21, 0.45);
  }

  .dialog-panel.info {
    border-color: rgba(59, 130, 246, 0.4);
  }

  .dialog-panel__header {
    @apply flex items-start justify-between gap-4;
  }

  .dialog-panel__title {
    @apply text-base font-semibold;
  }

  .dialog-panel__close {
    @apply inline-flex h-8 w-8 items-center justify-center rounded-full text-slate-500 transition-colors hover:bg-slate-200/80 hover:text-slate-800 focus:outline-none focus:ring-2 focus:ring-sky-300/60;
  }

  .dark .dialog-panel__close {
    @apply text-slate-300 hover:bg-slate-700/70 hover:text-slate-100;
  }

  .dialog-panel__close-icon {
    @apply h-4 w-4;
  }

  .dialog-panel__message {
    @apply mt-4 leading-relaxed;
    color: rgba(51, 65, 85, 0.9);
  }

  .dark .dialog-panel__message {
    @apply text-slate-200;
  }

  .dialog-panel__footer {
    @apply mt-6 flex justify-end;
  }

  .dialog-panel__action {
    @apply px-5 py-2 text-xs;
  }
}

@keyframes pulse {
  0% {
    box-shadow: 0 0 0 0 rgba(16, 185, 129, 0.6);
  }
  70% {
    box-shadow: 0 0 0 12px rgba(16, 185, 129, 0);
  }
  100% {
    box-shadow: 0 0 0 0 rgba(16, 185, 129, 0);
  }
}

.title-bar {
  @apply relative flex h-11 w-full items-center justify-between border-b px-5 backdrop-blur-xl;
  border-color: rgba(148, 163, 184, 0.25);
  background-image: linear-gradient(90deg, rgba(59, 130, 246, 0.18), rgba(14, 165, 233, 0));
}

.dark .title-bar {
  @apply border-white/10;
  background-image: linear-gradient(90deg, rgba(56, 189, 248, 0.2), rgba(59, 130, 246, 0));
}

.title-bar::after {
  content: '';
  position: absolute;
  inset: auto 0 -1px 0;
  height: 1px;
  background: linear-gradient(90deg, rgba(59, 130, 246, 0), rgba(59, 130, 246, 0.45), rgba(59, 130, 246, 0));
}

.title-bar__brand {
  @apply flex items-center gap-3 text-[11px] font-medium uppercase tracking-[0.32em];
  color: rgba(30, 64, 175, 0.75);
}

.dark .title-bar__brand {
  @apply text-slate-200/80;
}

.title-bar__logo {
  @apply relative flex h-7 w-7 items-center justify-center overflow-hidden rounded-xl border;
  border-color: rgba(59, 130, 246, 0.35);
  background: linear-gradient(135deg, rgba(191, 219, 254, 0.65), rgba(59, 130, 246, 0.12));
  box-shadow: inset 0 0 10px rgba(59, 130, 246, 0.25), 0 10px 25px rgba(148, 163, 184, 0.35);
}

.dark .title-bar__logo {
  @apply border-sky-400/40 bg-slate-900/70;
  box-shadow: inset 0 0 10px rgba(56, 189, 248, 0.4), 0 10px 25px rgba(8, 47, 73, 0.45);
}

.title-bar__logo::before,
.title-bar__logo::after {
  content: '';
  position: absolute;
  inset: 1px;
  border-radius: 10px;
}

.title-bar__logo::before {
  background: linear-gradient(135deg, rgba(59, 130, 246, 0.35), rgba(14, 165, 233, 0.12));
}

.title-bar__logo::after {
  background: radial-gradient(circle at 30% 30%, rgba(96, 165, 250, 0.6), transparent 55%);
  mix-blend-mode: screen;
}

.title-bar__logo-core {
  @apply relative block h-2 w-2 rounded-full;
  background: #0ea5e9;
  box-shadow: 0 0 12px rgba(56, 189, 248, 0.65);
}

.dark .title-bar__logo-core {
  @apply bg-sky-200;
  box-shadow: 0 0 12px rgba(186, 230, 253, 0.9);
}

.title-bar__text {
  @apply flex flex-col leading-tight;
}

.title-bar__text strong {
  @apply text-xs font-semibold tracking-[0.18em];
  color: rgba(30, 41, 59, 0.95);
}

.dark .title-bar__text strong {
  @apply text-slate-100;
}

.title-bar__text span {
  @apply text-[10px] uppercase tracking-[0.35em];
  color: rgba(71, 85, 105, 0.85);
}

.dark .title-bar__text span {
  @apply text-slate-400;
}

.title-bar__controls {
  @apply flex items-center gap-2;
}

.cursor-move {
  cursor: move;
}

.select-none {
  user-select: none;
}

.pointer-events-none {
  pointer-events: none;
}

.pointer-events-auto {
  pointer-events: auto;
}

.scrollbar-thin {
  scrollbar-width: thin;
  scrollbar-color: rgba(156, 163, 175, 0.5) transparent;
}

.scrollbar-thin::-webkit-scrollbar {
  width: 6px;
}

.scrollbar-thin::-webkit-scrollbar-track {
  background: transparent;
}

.scrollbar-thin::-webkit-scrollbar-thumb {
  background-color: rgba(156, 163, 175, 0.5);
  border-radius: 3px;
}

.scrollbar-thin::-webkit-scrollbar-thumb:hover {
  background-color: rgba(156, 163, 175, 0.7);
}
```

---

## 主题状态管理与切换组件

### `src/stores/theme.ts`

```ts
import { defineStore } from 'pinia'
import { computed, ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { isTauri } from '../utils/runtime'

type ThemeMode = 'light' | 'dark' | 'auto'
type SystemTheme = 'light' | 'dark'

const LOCAL_THEME_KEY = 'theme'

export const useThemeStore = defineStore('theme', () => {
  const theme = ref<ThemeMode>('auto')
  const systemTheme = ref<SystemTheme>('light')

  const isDark = computed(() => {
    if (theme.value === 'auto') {
      return systemTheme.value === 'dark'
    }
    return theme.value === 'dark'
  })

  const updateMetaThemeColor = () => {
    if (typeof document === 'undefined') {
      return
    }
    const meta = document.querySelector("meta[name='theme-color']") as HTMLMetaElement | null
    if (!meta) {
      return
    }
    meta.content = isDark.value ? '#111827' : '#f2f2f7'
  }

  const applyTheme = () => {
    if (typeof document === 'undefined') {
      return
    }
    const root = document.documentElement
    const body = document.body

    if (isDark.value) {
      root.classList.add('dark')
      root.classList.remove('light')
      body.classList.add('dark')
      body.classList.remove('light')
      root.style.colorScheme = 'dark'
      body.style.backgroundColor = '#111827'
    } else {
      root.classList.add('light')
      root.classList.remove('dark')
      body.classList.add('light')
      body.classList.remove('dark')
      root.style.colorScheme = 'light'
      body.style.backgroundColor = '#f2f2f7'
    }

    updateMetaThemeColor()
  }

  const detectSystemTheme = () => {
    if (typeof window === 'undefined') {
      return
    }
    const mediaQuery = window.matchMedia('(prefers-color-scheme: dark)')
    systemTheme.value = mediaQuery.matches ? 'dark' : 'light'

    mediaQuery.addEventListener('change', (event) => {
      systemTheme.value = event.matches ? 'dark' : 'light'
      if (theme.value === 'auto') {
        applyTheme()
      }
    })
  }

  const fetchInitialTheme = async () => {
    if (!isTauri()) {
      const stored = typeof window !== 'undefined' ? window.localStorage.getItem(LOCAL_THEME_KEY) : null
      if (stored === 'light' || stored === 'dark' || stored === 'auto') {
        theme.value = stored
      }
      applyTheme()
      return
    }

    try {
      const value = await invoke<ThemeMode>('tool_read_theme')
      theme.value = value
    } catch (error) {
      console.warn('读取主题失败，使用自动模式', error)
      theme.value = 'auto'
    }
    applyTheme()
  }

  const initialize = async () => {
    if (typeof window === 'undefined') {
      return
    }
    detectSystemTheme()
    await fetchInitialTheme()
  }

  const persistTheme = async (next: ThemeMode) => {
    if (typeof window !== 'undefined') {
      window.localStorage.setItem(LOCAL_THEME_KEY, next)
    }

    if (!isTauri()) {
      return
    }

    try {
      await invoke('save_config', { key: 'theme', value: next })
    } catch (error) {
      console.error('保存主题失败', error)
    }

    try {
      await invoke('sync_theme_env', { value: next })
    } catch (error) {
      console.error('同步主题环境变量失败', error)
    }
  }

  const setTheme = async (next: ThemeMode) => {
    theme.value = next
    applyTheme()

    await persistTheme(next)
  }

  return {
    theme,
    systemTheme,
    isDark,
    initialize,
    setTheme,
  }
})
```

### `src/components/ThemeToggle.vue`

```vue
<template>
  <button
    class="w-8 h-8 flex items-center justify-center rounded-lg bg-gray-100 dark:bg-gray-700 text-gray-600 dark:text-gray-300 hover:bg-gray-200 dark:hover:bg-gray-600 transition-colors"
    :title="`当前主题：${themeLabel}`"
    @click="toggleTheme"
  >
    <MoonIcon v-if="isDark" class="w-4 h-4" />
    <SunIcon v-else class="w-4 h-4" />
  </button>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { storeToRefs } from 'pinia'
import { MoonIcon, SunIcon } from '@heroicons/vue/24/outline'
import { useThemeStore } from '../stores/theme'

type ThemeMode = 'light' | 'dark' | 'auto'

const themeStore = useThemeStore()
const { isDark, theme } = storeToRefs(themeStore)

const themeLabel = computed(() => {
  switch (theme.value) {
    case 'dark':
      return '深色模式'
    case 'light':
      return '浅色模式'
    default:
      return '跟随系统'
  }
})

const toggleTheme = () => {
  const modes: ThemeMode[] = ['light', 'dark', 'auto']
  const currentIndex = modes.indexOf(theme.value)
  const next = modes[(currentIndex + 1) % modes.length]
  void themeStore.setTheme(next)
}
</script>
```

---

## 窗口标题栏与操作控件

### `src/components/TitleBar.vue`

```vue
<template>
  <header
    class="title-bar"
    data-tauri-drag-region
    @mousedown="startDragging"
  >
    <div class="title-bar__brand pointer-events-none">
      <span class="title-bar__logo" aria-hidden="true">
        <span class="title-bar__logo-core" />
      </span>
      <div class="title-bar__text">
        <strong>Unified Suite</strong>
        <span>Runtime Console</span>
      </div>
    </div>
    <div
      class="title-bar__controls pointer-events-auto"
      data-tauri-drag-region="false"
      @mousedown.stop
    >
      <ThemeToggle />
      <WindowControls />
    </div>
  </header>
</template>

<script setup lang="ts">
import { getCurrentWindow } from '@tauri-apps/api/window'
import ThemeToggle from './ThemeToggle.vue'
import WindowControls from './WindowControls.vue'

const appWindow = getCurrentWindow()

const startDragging = async (event: MouseEvent) => {
  if (event.button === 0) {
    try {
      await appWindow.startDragging()
    } catch (error) {
      console.error('Failed to start dragging:', error)
    }
  }
}
</script>
```

### `src/components/WindowControls.vue`

```vue
<template>
  <div class="flex items-center space-x-1">
    <button
      class="w-8 h-8 flex items-center justify-center rounded hover:bg-gray-200 dark:hover:bg-gray-700"
      @click.stop.prevent="minimize"
      @mousedown.stop
    >
      <MinusIcon class="w-4 h-4 text-gray-600 dark:text-gray-300" />
    </button>
    <button
      class="w-8 h-8 flex items-center justify-center rounded hover:bg-gray-200 dark:hover:bg-gray-700"
      @click.stop.prevent="toggleMaximize"
      @mousedown.stop
    >
      <Squares2X2Icon
        v-if="!isMaximized"
        class="w-4 h-4 text-gray-600 dark:text-gray-300"
      />
      <Squares2X2Icon
        v-else
        class="w-4 h-4 text-gray-600 dark:text-gray-300 rotate-45"
      />
    </button>
    <button
      class="w-8 h-8 flex items-center justify-center rounded hover:bg-red-500 hover:text-white"
      @click.stop.prevent="closeWindow"
      @mousedown.stop
    >
      <XMarkIcon class="w-4 h-4" />
    </button>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue'
import type { UnlistenFn } from '@tauri-apps/api/event'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { MinusIcon, Squares2X2Icon, XMarkIcon } from '@heroicons/vue/24/outline'

const appWindow = getCurrentWindow()
const isMaximized = ref(false)
const listeners: UnlistenFn[] = []

type WindowApiWithOptionalEvents = typeof appWindow & {
  onMaximize?: (handler: () => void) => Promise<UnlistenFn>
  onUnmaximize?: (handler: () => void) => Promise<UnlistenFn>
}

const windowApi = appWindow as WindowApiWithOptionalEvents

const refreshState = async () => {
  try {
    isMaximized.value = await appWindow.isMaximized()
  } catch (error) {
    console.error('Failed to determine maximize state:', error)
  }
}

onMounted(async () => {
  await refreshState()

  try {
    listeners.push(await appWindow.onResized(refreshState))

    if (typeof windowApi.onMaximize === 'function') {
      listeners.push(
        await windowApi.onMaximize(() => {
          isMaximized.value = true
        }),
      )
    }

    if (typeof windowApi.onUnmaximize === 'function') {
      listeners.push(
        await windowApi.onUnmaximize(() => {
          isMaximized.value = false
        }),
      )
    }
  } catch (error) {
    console.error('Failed to register window listeners:', error)
  }
})

onUnmounted(() => {
  listeners.forEach((unlisten) => {
    try {
      unlisten()
    } catch (error) {
      console.warn('Failed to clean up window listener:', error)
    }
  })
  listeners.length = 0
})

const minimize = async () => {
  try {
    await appWindow.minimize()
  } catch (error) {
    console.error('Failed to minimize window:', error)
  }
}

const toggleMaximize = async () => {
  try {
    const maximized = await appWindow.isMaximized()
    if (maximized) {
      await appWindow.unmaximize()
    } else {
      await appWindow.maximize()
    }
    await refreshState()
  } catch (error) {
    console.error('Failed to toggle maximize:', error)
  }
}

const closeWindow = async () => {
  try {
    await appWindow.close()
  } catch (error) {
    console.error('Failed to close window:', error)
  }
}
</script>
```

### `src/components/MirrorSelectDropdown.vue`

```vue
<template>
  <div ref="root" class="mirror-select__control">
    <span class="mirror-select__icon" aria-hidden="true">
      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
        <circle cx="12" cy="12" r="6.75" opacity="0.4" />
        <path
          d="M4.75 12h14.5M12 4.75c2.5 2.15 2.5 10.35 0 14.5M12 4.75c-2.5 2.15-2.5 10.35 0 14.5"
          stroke-linecap="round"
        />
      </svg>
    </span>

    <button
      ref="trigger"
      type="button"
      class="mirror-select__input"
      :class="{ 'mirror-select__input--disabled': disabled }"
      :data-open="isOpen ? 'true' : null"
      :aria-expanded="isOpen ? 'true' : 'false'"
      aria-haspopup="listbox"
      :aria-controls="menuId"
      :aria-labelledby="computedLabelId + ' ' + valueId"
      :aria-disabled="disabled ? 'true' : 'false'"
      @click="handleTriggerClick"
      @keydown="handleTriggerKeydown"
    >
      <span :id="valueId" class="mirror-select__value" :class="{ 'mirror-select__value--empty': !selectedOption }">
        <template v-if="selectedOption">
          <span class="mirror-select__value-label">{{ selectedOption.label }}</span>
          <span class="mirror-select__value-url">{{ selectedOption.value }}</span>
        </template>
        <template v-else>
          <span class="mirror-select__placeholder">暂无可用镜像</span>
        </template>
      </span>
    </button>

    <span class="mirror-select__chevron" aria-hidden="true">
      <svg viewBox="0 0 20 20" fill="none" stroke="currentColor" stroke-width="1.5">
        <path d="M6 8l4 4 4-4" stroke-linecap="round" stroke-linejoin="round" />
      </svg>
    </span>
    <span class="mirror-select__shine" aria-hidden="true" />
    <span class="mirror-select__border" aria-hidden="true" />

    <transition name="mirror-select__dropdown-transition">
      <ul
        v-if="isOpen"
        :id="menuId"
        class="mirror-select__dropdown"
        role="listbox"
        :aria-activedescendant="activeDescendantId"
        :aria-labelledby="computedLabelId"
        tabindex="-1"
      >
        <template v-if="options.length">
          <li
            v-for="(option, index) in options"
            :id="getOptionId(index)"
            :key="option.value"
            :ref="(el) => setOptionRef(el, index)"
            role="option"
            class="mirror-select__option"
            :class="{
              'mirror-select__option--active': index === highlightedIndex,
              'mirror-select__option--selected': option.value === modelValue,
            }"
            :aria-selected="option.value === modelValue"
            @mouseenter="highlight(index)"
            @mousedown.prevent
            @click="selectOption(option)"
          >
            <span class="mirror-select__option-label">{{ option.label }}</span>
            <span class="mirror-select__option-url">{{ option.value }}</span>
          </li>
        </template>
        <li v-else class="mirror-select__empty">暂无可用镜像源</li>
      </ul>
    </transition>
  </div>
</template>

<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, onBeforeUpdate, onMounted, ref, watch } from 'vue'

type MirrorOption = {
  label: string
  value: string
}

const props = defineProps<{
  modelValue: string
  options: MirrorOption[]
  disabled?: boolean
  labelId?: string
}>()

const emit = defineEmits<{
  (e: 'update:modelValue', value: string): void
}>()

const instanceId = `mirror-select-${Math.random().toString(36).slice(2, 10)}`
const menuId = `${instanceId}-menu`
const valueId = `${instanceId}-value`

const computedLabelId = computed(() => props.labelId ?? `${instanceId}-label`)

const isOpen = ref(false)
const highlightedIndex = ref(-1)
const trigger = ref<HTMLElement | null>(null)
const root = ref<HTMLElement | null>(null)
const optionRefs = ref<HTMLElement[]>([])

const options = computed(() => props.options ?? [])

const selectedOption = computed(() => options.value.find((option) => option.value === props.modelValue) ?? null)

const activeDescendantId = computed(() =>
  highlightedIndex.value >= 0 ? getOptionId(highlightedIndex.value) : undefined,
)

function getOptionId(index: number) {
  return `${instanceId}-option-${index}`
}

function setOptionRef(el: Element | null, index: number) {
  if (!el) {
    return
  }
  optionRefs.value[index] = el as HTMLElement
}

function selectedIndex() {
  return options.value.findIndex((option) => option.value === props.modelValue)
}

function openDropdown() {
  if (props.disabled || !options.value.length) {
    return
  }
  isOpen.value = true
  nextTick(() => {
    const index = selectedIndex()
    if (index >= 0) {
      highlight(index)
    } else if (options.value.length) {
      highlight(0)
    }
  })
}

function closeDropdown(focusTrigger = false) {
  if (!isOpen.value) {
    return
  }
  isOpen.value = false
  highlightedIndex.value = -1
  if (focusTrigger) {
    nextTick(() => trigger.value?.focus())
  }
}

function handleTriggerClick() {
  if (props.disabled) {
    return
  }
  if (isOpen.value) {
    closeDropdown(true)
  } else {
    openDropdown()
  }
}

function handleTriggerKeydown(event: KeyboardEvent) {
  if (props.disabled) {
    return
  }

  switch (event.key) {
    case 'ArrowDown':
    case 'Down':
      event.preventDefault()
      if (!isOpen.value) {
        openDropdown()
      }
      moveHighlight(1)
      break
    case 'ArrowUp':
    case 'Up':
      event.preventDefault()
      if (!isOpen.value) {
        openDropdown()
      }
      moveHighlight(-1)
      break
    case 'Home':
      event.preventDefault()
      if (!isOpen.value) {
        openDropdown()
      }
      highlight(0)
      break
    case 'End':
      event.preventDefault()
      if (!isOpen.value) {
        openDropdown()
      }
      highlight(options.value.length - 1)
      break
    case 'Enter':
    case ' ':
    case 'Spacebar':
      event.preventDefault()
      if (!isOpen.value) {
        openDropdown()
      } else if (highlightedIndex.value >= 0) {
        const option = options.value[highlightedIndex.value]
        if (option) {
          selectOption(option)
        }
      }
      break
    case 'Escape':
      event.preventDefault()
      closeDropdown(true)
      break
    case 'Tab':
      closeDropdown()
      break
  }
}

function moveHighlight(step: number) {
  if (!options.value.length) {
    highlightedIndex.value = -1
    return
  }

  let nextIndex = highlightedIndex.value
  if (nextIndex === -1) {
    const selected = selectedIndex()
    nextIndex = selected >= 0 ? selected : step > 0 ? 0 : options.value.length - 1
  } else {
    nextIndex = nextIndex + step
  }

  if (nextIndex < 0) {
    nextIndex = options.value.length - 1
  } else if (nextIndex >= options.value.length) {
    nextIndex = 0
  }

  highlight(nextIndex)
}

function highlight(index: number) {
  if (!options.value.length) {
    highlightedIndex.value = -1
    return
  }
  const clampedIndex = Math.min(Math.max(index, 0), options.value.length - 1)
  highlightedIndex.value = clampedIndex
  nextTick(() => {
    const el = optionRefs.value[clampedIndex]
    el?.scrollIntoView({ block: 'nearest' })
  })
}

function selectOption(option: MirrorOption) {
  emit('update:modelValue', option.value)
  closeDropdown(true)
}

function handleClickOutside(event: MouseEvent) {
  if (!root.value) {
    return
  }

  const target = event.target as Node | null
  if (target && root.value.contains(target)) {
    return
  }
  closeDropdown()
}

watch(
  () => props.modelValue,
  () => {
    if (!isOpen.value) {
      return
    }
    const index = selectedIndex()
    if (index >= 0) {
      highlight(index)
    }
  },
)

watch(
  () => props.disabled,
  (disabled) => {
    if (disabled) {
      closeDropdown()
    }
  },
)

onMounted(() => {
  document.addEventListener('mousedown', handleClickOutside)
})

onBeforeUnmount(() => {
  document.removeEventListener('mousedown', handleClickOutside)
})

onBeforeUpdate(() => {
  optionRefs.value = []
})
</script>
```

---

## 主要页面结构

### `src/App.vue`

```vue
<template>
  <div class="app-shell">
    <span class="app-shell__glow app-shell__glow--primary" aria-hidden="true" />
    <span class="app-shell__glow app-shell__glow--secondary" aria-hidden="true" />
    <TitleBar />
    <main class="app-panel">
      <header class="panel-header">
        <div>
          <h1 class="panel-title">运行时环境配置助手</h1>
          <p class="panel-subtitle">一键侦测并智能修复运行时环境，快速接入本地高可用镜像源。</p>
        </div>
      </header>

      <section class="panel-section">
        <h2 class="panel-section__title">状态诊断</h2>
        <div class="status-grid">
          <article
            v-for="item in statusLines"
            :key="item.label"
            class="status-card"
            :class="item.status.ok ? 'status-card--ok' : 'status-card--error'"
          >
            <div class="status-card__icon" aria-hidden="true">
              <span class="status-card__beam" />
              <span class="status-card__dot" />
            </div>
            <div>
              <p class="status-card__label">{{ item.label }}</p>
              <p class="status-card__message">{{ item.status.message }}</p>
              <p v-if="item.status.detail" class="status-card__detail">{{ item.status.detail }}</p>
            </div>
          </article>
        </div>
      </section>

      <section class="panel-section panel-section--mirrors">
        <div class="panel-section__title-wrap">
          <h2 class="panel-section__title">镜像智能切换</h2>
          <p class="panel-section__hint">自适配可信镜像，保持依赖分发稳定。</p>
        </div>
        <div class="mirror-select">
          <label :id="mirrorSelectLabelId" class="mirror-select__label">选择默认包管理镜像源</label>
          <MirrorSelectDropdown
            v-model="selectedMirror"
            :options="mirrorOptions"
            :disabled="initializing"
            :label-id="mirrorSelectLabelId"
          />
          <p class="mirror-select__note">系统将写入全局配置并同步可信主机列表。</p>
        </div>
      </section>

      <footer class="panel-footer">
        <dl class="panel-meta">
          <div>
            <dt>当前运行时路径</dt>
            <dd>{{ status?.pythonPath ?? '未检测到' }}</dd>
          </div>
          <div>
            <dt>可执行脚本目录</dt>
            <dd>{{ status?.scriptsPath ?? '未检测到' }}</dd>
          </div>
          <div>
            <dt>当前镜像</dt>
            <dd>{{ status?.pipMirrorConfigured.currentMirror ?? '未检测到' }}</dd>
          </div>
        </dl>
        <button
          class="btn-primary panel-action"
          :disabled="initializing || loading"
          @click="handleInitialize"
        >
          <span v-if="initializing" class="panel-action__loading">
            <svg class="panel-action__spinner" viewBox="0 0 24 24" fill="none">
              <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4" />
              <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8v4a4 4 0 00-4 4H4z" />
            </svg>
            <span>正在初始化...</span>
          </span>
          <span v-else>开始初始化</span>
        </button>
      </footer>

      <Transition name="dialog">
        <div
          v-if="dialog.visible"
          class="dialog-backdrop"
          role="alertdialog"
          aria-modal="true"
          aria-labelledby="dialog-title"
        >
          <article class="dialog-panel" :class="dialog.type">
            <header class="dialog-panel__header">
              <h3 id="dialog-title" class="dialog-panel__title">{{ dialog.title }}</h3>
              <button class="dialog-panel__close" type="button" @click="closeDialog" aria-label="关闭提示">
                <svg class="dialog-panel__close-icon" viewBox="0 0 20 20" fill="none" aria-hidden="true">
                  <path d="M6 6l8 8M14 6l-8 8" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" />
                </svg>
              </button>
            </header>
            <p class="dialog-panel__message">{{ dialog.message }}</p>
            <footer class="dialog-panel__footer">
              <button class="btn-primary dialog-panel__action" type="button" @click="closeDialog">
                我知道了
              </button>
            </footer>
          </article>
        </div>
      </Transition>
    </main>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, reactive, ref } from 'vue'
import MirrorSelectDropdown from './components/MirrorSelectDropdown.vue'
import TitleBar from './components/TitleBar.vue'
import { fetchPythonEnvStatus, initializePythonEnvironment, type PythonEnvStatus } from './services/pythonEnv'

const status = ref<PythonEnvStatus | null>(null)
const loading = ref(true)
const initializing = ref(false)
const selectedMirror = ref('https://pypi.tuna.tsinghua.edu.cn/simple')
const dialog = reactive({
  visible: false,
  title: '',
  message: '',
  type: 'info' as 'success' | 'error' | 'info' | 'warning',
})

const mirrorSelectLabelId = 'mirror-select-label'

const mirrorOptions = computed(() => status.value?.mirrorCandidates ?? [
  {
    label: '清华大学 TUNA',
    value: 'https://pypi.tuna.tsinghua.edu.cn/simple',
    trustedHosts: ['pypi.tuna.tsinghua.edu.cn'],
  },
])

const statusLines = computed(() => {
  if (!status.value) {
    return []
  }
  return [
    { label: '当前目录检测', status: status.value.pythonPresent },
    { label: '运行时环境变量', status: status.value.pythonEnvVar },
    { label: 'PATH 中的运行时目录', status: status.value.pathConfigured },
    { label: 'PATH 中的脚本目录', status: status.value.scriptsConfigured },
    { label: '镜像配置', status: status.value.pipMirrorConfigured },
  ]
})

const openDialog = (item: { title: string; message: string; type?: 'success' | 'error' | 'info' | 'warning' }) => {
  dialog.title = item.title
  dialog.message = item.message
  dialog.type = item.type ?? 'info'
  dialog.visible = true
}

const closeDialog = () => {
  dialog.visible = false
}

const synchronizeMirrorSelection = () => {
  const current = status.value?.pipMirrorConfigured.currentMirror
  if (current && mirrorOptions.value.some((item) => item.value === current)) {
    selectedMirror.value = current
  } else if (mirrorOptions.value.length > 0) {
    selectedMirror.value = mirrorOptions.value[0].value
  }
}

const loadStatus = async () => {
  loading.value = true
  try {
    status.value = await fetchPythonEnvStatus()
    synchronizeMirrorSelection()
  } catch (error) {
    console.error(error)
    openDialog({
      title: '状态读取失败',
      message: error instanceof Error ? error.message : String(error),
      type: 'error',
    })
  } finally {
    loading.value = false
  }
}

const handleInitialize = async () => {
  if (!status.value) {
    return
  }
  initializing.value = true
  try {
    status.value = await initializePythonEnvironment({ mirror: selectedMirror.value })
    synchronizeMirrorSelection()
    openDialog({
      title: '初始化完成',
      message: '运行时环境变量与国内镜像已配置。',
      type: 'success',
    })
  } catch (error) {
    console.error(error)
    openDialog({
      title: '初始化失败',
      message: error instanceof Error ? error.message : String(error),
      type: 'error',
    })
  } finally {
    initializing.value = false
  }
}

onMounted(async () => {
  await loadStatus()
})
</script>
```

---

## 离线密钥服务（前端）

### `src/services/offlineLicense.ts`

```ts
import { invoke } from '@tauri-apps/api/core'

export interface OfflineLicensePayload {
  userId: number
  username: string
  email: string
  deviceId: string
  expiresAt: string
  issuedAt: string
}

export interface OfflineKeyValidationResult {
  isValid: boolean
  reason?: string
  expiresAt?: string
  payload?: OfflineLicensePayload
}

const isTauriAvailable = () =>
  typeof window !== 'undefined' && Boolean((window as any).__TAURI__)

const logValidationSuccess = (result: OfflineKeyValidationResult) => {
  const payload = result.payload
  if (payload) {
    const expiresAt = result.expiresAt ?? payload.expiresAt ?? '未知'
    console.info(
      `离线密钥校验成功：用户 ${payload.username} (ID: ${payload.userId})，设备 ${payload.deviceId}，到期时间 ${expiresAt}`,
    )
  } else {
    console.info('离线密钥校验成功')
  }
}

const showAuthError = async (content: string) => {
  if (!isTauriAvailable()) {
    alert(content)
    return
  }

  const dialog = (window as any).__TAURI__?.dialog
  if (dialog?.message) {
    await dialog.message(content, { title: '认证失败', type: 'error' })
  } else {
    alert(content)
  }
}

const exitApp = async () => {
  if (!isTauriAvailable()) {
    return
  }
  const processApi = (window as any).__TAURI__?.process
  if (processApi?.exit) {
    await processApi.exit(0)
  }
}

export class OfflineLicenseAPI {
  static async validateOfflineKey(): Promise<OfflineKeyValidationResult> {
    return await invoke<OfflineKeyValidationResult>('validate_offline_key')
  }

  static async getDeviceId(): Promise<string> {
    return await invoke<string>('get_device_id')
  }
}

export const ensureOfflineLicense = async (): Promise<boolean> => {
  if (!isTauriAvailable()) {
    return true
  }

  try {
    const result = await OfflineLicenseAPI.validateOfflineKey()
    if (!result.isValid) {
      const reason = result.reason ?? '离线密钥无效'
      const expiresAt = result.expiresAt ? `\n到期时间：${result.expiresAt}` : ''
      const message = `认证失败：${reason}${expiresAt}`
      console.error(`离线密钥校验失败：${reason}${expiresAt}`)
      await showAuthError(message)
      await exitApp()
      return false
    }
    logValidationSuccess(result)
    return true
  } catch (error) {
    const detail = error instanceof Error ? error.message : String(error)
    const message = `认证失败：${detail}`
    console.error(`离线密钥校验失败：${detail}`)
    await showAuthError(message)
    await exitApp()
    return false
  }
}
```

---

## 应用入口初始化流程

### `src/main.ts`

```ts
import { createApp } from 'vue'
import { createPinia } from 'pinia'
import App from './App.vue'
import './styles/theme.css'
import { useThemeStore } from './stores/theme'
import { ensureOfflineLicense } from './services/offlineLicense'

const bootstrap = async () => {
  const licensed = await ensureOfflineLicense()
  if (!licensed) {
    console.error('离线密钥认证失败，已终止应用启动流程')
    return
  }

  const app = createApp(App)
  const pinia = createPinia()
  app.use(pinia)

  const themeStore = useThemeStore(pinia)
  await themeStore.initialize()

  app.mount('#app')
}

void bootstrap()
```

---

## 后端配置与主题持久化

### `src-tauri/src/lib.rs`

```rust
mod config;
mod device;
mod offline_key;
mod python_env;
mod storage;

use config::FileWriteLock;
use tauri::Manager;

use std::process;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(FileWriteLock::default())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            offline_key::validate_offline_key,
            offline_key::get_device_id,
            python_env::get_python_environment_status,
            python_env::initialize_python_environment,
            config::tool_read_theme,
            config::save_config,
            config::sync_theme_env
        ])
        .setup(|app| {
            let handle = app.handle();
            config::ensure_config_dir(&handle)?;

            let lock_state: tauri::State<'_, FileWriteLock> = app.state();
            let lock = lock_state.0.clone();
            drop(lock_state);

            let validation = tauri::async_runtime::block_on(async {
                offline_key::validate_from_env(&lock).await
            });

            if !validation.is_valid {
                let reason = validation
                    .reason
                    .unwrap_or_else(|| String::from("离线密钥认证失败"));
                eprintln!("离线密钥认证失败：{reason}");
                process::exit(1);
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running application");
}
```

### `src-tauri/src/config.rs`

```rust
use anyhow::{anyhow, Context, Result};
use serde_json::{json, Map, Value};
use std::{collections::HashMap, fs, path::PathBuf, sync::Arc};
use tauri::{Emitter, Manager};
use tokio::sync::Mutex;

pub const APP_THEME_ENV_NAME: &str = "APP_THEME_MODE";
pub const APP_OFFLINE_KEY_ENV_NAME: &str = "APP_OFFLINE_KEY_PATH";
const CONFIG_FILE_NAME: &str = "settings.json";

#[derive(Clone)]
pub struct FileWriteLock(pub Arc<Mutex<()>>);

impl Default for FileWriteLock {
    fn default() -> Self {
        Self(Arc::new(Mutex::new(())))
    }
}

pub fn ensure_config_dir(app: &tauri::AppHandle) -> Result<()> {
    let path = app
        .path()
        .app_config_dir()
        .map_err(|err| anyhow!("无法定位配置目录: {err}"))?;
    fs::create_dir_all(&path).with_context(|| format!("无法创建配置目录: {}", path.display()))?;
    Ok(())
}

fn config_path(app: &tauri::AppHandle) -> Result<PathBuf> {
    ensure_config_dir(app)?;
    let dir = app
        .path()
        .app_config_dir()
        .map_err(|err| anyhow!("无法定位配置目录: {err}"))?;
    Ok(dir.join(CONFIG_FILE_NAME))
}

fn load_config(path: &PathBuf) -> Result<Map<String, Value>> {
    if !path.exists() {
        return Ok(Map::new());
    }

    let content = fs::read_to_string(path)
        .with_context(|| format!("读取配置文件失败: {}", path.display()))?;
    if content.trim().is_empty() {
        return Ok(Map::new());
    }

    let map: Map<String, Value> = serde_json::from_str(&content)
        .with_context(|| format!("解析配置文件失败: {}", path.display()))?;
    Ok(map)
}

fn save_config_map(path: &PathBuf, map: &Map<String, Value>) -> Result<()> {
    let content = serde_json::to_string_pretty(map)?;
    fs::write(path, content).with_context(|| format!("写入配置文件失败: {}", path.display()))?;
    Ok(())
}

fn with_lock<T>(lock: &FileWriteLock, task: impl FnOnce() -> Result<T>) -> Result<T> {
    let _guard = lock.0.blocking_lock();
    task()
}

fn sanitize_theme(theme: &str) -> &str {
    match theme {
        "light" | "dark" | "auto" => theme,
        _ => "auto",
    }
}

#[tauri::command]
pub fn tool_read_theme(
    app: tauri::AppHandle,
    lock: tauri::State<FileWriteLock>,
) -> Result<String, String> {
    if let Ok(value) = std::env::var(APP_THEME_ENV_NAME) {
        return Ok(sanitize_theme(value.trim()).to_string());
    }

    let path = config_path(&app).map_err(|err| err.to_string())?;
    let lock_ref = lock.inner();
    let result = with_lock(lock_ref, || {
        let mut map = load_config(&path)?;
        if let Some(value) = map.remove("theme") {
            Ok(sanitize_theme(value.as_str().unwrap_or("auto")).to_string())
        } else {
            Ok(String::from("auto"))
        }
    });

    result.map_err(|err| err.to_string())
}

#[tauri::command]
pub fn save_config(
    app: tauri::AppHandle,
    key: String,
    value: String,
    lock: tauri::State<FileWriteLock>,
) -> Result<(), String> {
    let should_emit_theme = key == "theme";
    let path = config_path(&app).map_err(|err| err.to_string())?;

    with_lock(lock.inner(), || {
        let mut map = load_config(&path)?;
        map.insert(key.clone(), json!(value));
        save_config_map(&path, &map)?;
        Ok(())
    })
    .map_err(|err| err.to_string())?;

    if should_emit_theme {
        emit_theme_update(&app);
    }

    Ok(())
}

pub fn emit_theme_update(app: &tauri::AppHandle) {
    let _ = app.emit("theme-changed", HashMap::<String, String>::new());
}

#[tauri::command]
pub fn sync_theme_env(value: String) -> Result<(), String> {
    sync_theme_env_impl(&value).map_err(|err| err.to_string())
}

#[cfg(target_os = "windows")]
fn sync_theme_env_impl(value: &str) -> Result<()> {
    use windows::core::w;
    use windows::Win32::Foundation::{LPARAM, WPARAM};
    use windows::Win32::UI::WindowsAndMessaging::{
        SendMessageTimeoutW, HWND_BROADCAST, SMTO_ABORTIFHUNG, WM_SETTINGCHANGE,
    };
    use winreg::enums::HKEY_CURRENT_USER;
    use winreg::RegKey;

    let sanitized = sanitize_theme(value);
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let (env, _) = hkcu.create_subkey("Environment")?;
    env.set_value(APP_THEME_ENV_NAME, &sanitized)?;

    unsafe {
        let param = w!("Environment");
        let _ = SendMessageTimeoutW(
            HWND_BROADCAST,
            WM_SETTINGCHANGE,
            WPARAM::default(),
            LPARAM(param.as_ptr() as isize),
            SMTO_ABORTIFHUNG,
            5000,
            None,
        );
    }

    Ok(())
}

#[cfg(not(target_os = "windows"))]
fn sync_theme_env_impl(value: &str) -> Result<()> {
    std::env::set_var(APP_THEME_ENV_NAME, sanitize_theme(value));
    Ok(())
}
```

## 离线密钥认证核心逻辑（后端）

### `src-tauri/src/offline_key.rs`

```rust
use aes_gcm::aead::{Aead, KeyInit};
use aes_gcm::{Aes256Gcm, Nonce};
use base64::{engine::general_purpose, Engine as _};
use chrono::{DateTime, Utc};
use once_cell::sync::Lazy;
use rand::rngs::OsRng;
use rand::RngCore;
use rsa::pkcs8::DecodePrivateKey;
use rsa::{Oaep, RsaPrivateKey};
use serde::Serialize;
use serde_json::Value;
use sha2::Sha256;
use std::convert::TryFrom;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::Mutex;

use crate::config::{FileWriteLock, APP_OFFLINE_KEY_ENV_NAME};
use crate::device;

const OFFLINE_RSA_PRIVATE_KEY: &str = r"-----BEGIN PRIVATE KEY-----
MIIEvgIBADANBgkqhkiG9w0BAQEFAASCBKgwggSkAgEAAoIBAQDVKT08cxtJyhsx
l9A0+6o7PR5EN12E/gaVXSInY/g9GgTeCcaXBm7FifosFCaVGcvmjJrhxTaNsp/7
VM+S7jJVfmIYuVnwhhGdis5iJlG8fL2ekG1c1HtShxK7vcrhv8lDrc3zEIbX1v6r
9sy4T8gZH8peg6dnzmMKHRPhXGscr2SIHn3xsdkP/kCY+4mOsRLdV0IESho0BsNC
W4smTp4lx9zZKM9Q6DNF62B/2Gd4v+vTixogouQVbSJspTg/AbRx+snZbvX1Fe2d
I1pFvosWO/rh/K2Wt4PW9KUoQOQXt1WUWIQv5+4FQI82zcRR0BuUf4bfPnUy64ms
b8cfUA+1AgMBAAECggEAAP1hTitPG/u3iLR3KIeQfq0dmpZ8ED8KS0T2xtgktxUT
1IwKlui1NIv2rROflgu/DcZe0EDhtJhPlz79EX71W3x+mZBvR0x8dJ1QfzTZy5El
a+5m4MIpj1V1tgzoeYf4K5yV+RzgWuUmCiCZc9Crqo509We+vI5JX1g2zPMqgSAe
CEQnPPvXPIPrMeo8fHJuW7OLytuGCzIogYY9vdPeTde7tBn/5FOz9tNhqSxNR1Ru
2vNvgBsacWNZ0x5aeXLj698RMNm92Ny/ytOdvcYRBtt7677FKZ3GpnDMeB8uHyKF
3MHm0n1uaXpYJzaYrAPLNFoPt4uWp9uCtCHY5YhbQQKBgQDt/XGFepU/moQXTpKE
aVKxXtqbqvgEJBdlq9xZ6Z4I3yvVxiVNC08ponHwGUI8XJ/ATR56DiKfIrEyx72o
RL39AoFaQ6XHwTnk1wgZ2M10HZksFX4SLCYewWesoOwzPYwVu6kLBDZINIw4qdzv
Et23dLsec3bjDEwgdlVIuBdxlQKBgQDlSsmcO3qUroLwkHSKq5d6lGZA0HWEADcm
fqb/B4fAtTsG7aaw7mFIiEbkvzG4MdH+lpx+6pOCZPPEwI6RmIjgb9mxpv5Wy0XL
SOPxYsAjlhdYfZ88ZTueyTBDnd1k1VbLKhC4hNuCEgMI3LfVJv/3GXp0gJu5bGAB
6kQgFHTdoQKBgBQyoEHNx4DgYjmAJ5spPSVkgXUYq3fegEXWshrHYuwp1JSN/nht
b0h/SuAvpJlu2vf9E4sUTAfpb9R5czUmsGEap1O7zgQH+BvdzAg1iCpEoM1G/a4Z
JRsTGvNhrOokXREzHgObVegG3aepcuCvXzXEqGTLM9nNH2DZ6h8D0KmJAoGBAMpZ
Ucragrcruspp8S9fdvLqe8K/NLYlKoaCRwXRs2/RgCIBIJYMCTZlbYr5X/tZnCS8
7abjhQIR7T65YBgFMOZATzGEWfhms1VPIjooF8BP+JJTam92N0NN8ZX6fyM5UrtA
iDkOplkHZD4x6tnk7Qc4KOUfik384k1OXIijBO+BAoGBANmeLjg9ZQicQgNFHUF2
IHXWLXk7UXEGIGc94fpBs3XJv3Gs33STpdrUZ7HpMxeaNyKT7o3h0gMRfCAWdE+Y
6+0pcY3LO8xt47mM0fJ4rmO3UtPtRFCB4zfl5LLRXTxMWlDGGeMpkXnTDyepdUer
ngynrceK8F5b7rIPg7tmoFji
-----END PRIVATE KEY-----";
const OFFLINE_AES_KEY_B64: &str = "94/AR7dd8gIstLEXp3LCs865DptiMKlh8nLjjDEcO40=";
const OFFLINE_KEY_SEPARATOR: &str = "|||";

static PRIVATE_KEY: Lazy<RsaPrivateKey> = Lazy::new(|| {
    RsaPrivateKey::from_pkcs8_pem(OFFLINE_RSA_PRIVATE_KEY)
        .expect("failed to parse offline RSA private key")
});

static AES_KEY: Lazy<[u8; 32]> = Lazy::new(|| {
    let bytes = general_purpose::STANDARD
        .decode(OFFLINE_AES_KEY_B64)
        .expect("failed to decode offline AES key");
    bytes
        .try_into()
        .expect("invalid offline AES key length: expected 32 bytes")
});

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OfflineLicensePayload {
    pub user_id: u32,
    pub username: String,
    pub email: String,
    pub device_id: String,
    pub expires_at: String,
    pub issued_at: String,
}

impl OfflineLicensePayload {
    fn from_value(value: &Value) -> Result<Self, OfflineKeyError> {
        let map = value
            .as_object()
            .ok_or_else(|| OfflineKeyError::InvalidData("离线密钥数据不完整".into()))?;

        let user_id = Self::read_user_id(map, &["userId", "user_id"])?;
        let username = Self::read_string(map, &["username"])?;
        let email = Self::read_string(map, &["email"])?;
        let device_id = Self::read_string(map, &["deviceId", "device_id"])?;
        let expires_at = Self::read_string(map, &["expiresAt", "expires_at"])?;
        let issued_at = Self::read_string(map, &["issuedAt", "issued_at"])?;

        Ok(Self {
            user_id,
            username,
            email,
            device_id,
            expires_at,
            issued_at,
        })
    }
```
    fn read_user_id(
        map: &serde_json::Map<String, Value>,
        keys: &[&str],
    ) -> Result<u32, OfflineKeyError> {
        let value = Self::read_value(map, keys)?;
        if let Some(number) = value.as_u64() {
            return u32::try_from(number)
                .map_err(|_| OfflineKeyError::InvalidData("离线密钥数据不完整".into()));
        }

        if let Some(number) = value.as_i64() {
            if number < 0 {
                return Err(OfflineKeyError::InvalidData("离线密钥数据不完整".into()));
            }
            return u32::try_from(number as u64)
                .map_err(|_| OfflineKeyError::InvalidData("离线密钥数据不完整".into()));
        }

        if let Some(text) = value.as_str() {
            return text
                .trim()
                .parse::<u32>()
                .map_err(|_| OfflineKeyError::InvalidData("离线密钥数据不完整".into()));
        }

        Err(OfflineKeyError::InvalidData("离线密钥数据不完整".into()))
    }

    fn read_string(
        map: &serde_json::Map<String, Value>,
        keys: &[&str],
    ) -> Result<String, OfflineKeyError> {
        let value = Self::read_value(map, keys)?;
        if let Some(text) = value.as_str() {
            if text.trim().is_empty() {
                return Err(OfflineKeyError::InvalidData("离线密钥数据不完整".into()));
            }
            return Ok(text.to_owned());
        }
        Err(OfflineKeyError::InvalidData("离线密钥数据不完整".into()))
    }

    fn read_value<'a>(
        map: &'a serde_json::Map<String, Value>,
        keys: &[&str],
    ) -> Result<&'a Value, OfflineKeyError> {
        for key in keys {
            if let Some(value) = map.get(*key) {
                return Ok(value);
            }
        }
        Err(OfflineKeyError::InvalidData("离线密钥数据不完整".into()))
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OfflineKeyValidationResult {
    pub is_valid: bool,
    pub reason: Option<String>,
    pub expires_at: Option<String>,
    pub payload: Option<OfflineLicensePayload>,
}

#[derive(Debug, Error)]
enum OfflineKeyError {
    #[error("离线密钥格式无效")]
    InvalidFormat,
    #[error("离线密钥数据无效: {0}")]
    InvalidData(String),
    #[error("离线密钥加解密失败: {0}")]
    Crypto(String),
    #[error("文件操作失败: {0}")]
    Io(#[from] std::io::Error),
}

fn log_validation_failure(reason: &str, payload: Option<&OfflineLicensePayload>) {
    eprintln!("离线密钥校验失败: {reason}");

    if let Some(payload) = payload {
        eprintln!(
            "密钥绑定信息 -> 用户ID: {}, 用户名: {}, 设备ID: {}, 到期时间: {}",
            payload.user_id, payload.username, payload.device_id, payload.expires_at
        );
    }

    match device::get_device_info() {
        Ok(info) => {
            eprintln!(
                "当前设备信息 -> 设备ID: {}, 设备名称: {}, 操作系统: {}, 架构: {}, 主机名: {}, 信息创建时间: {}",
                info.device_id,
                info.device_name,
                info.os,
                info.arch,
                info.hostname,
                info.created_at.to_rfc3339()
            );
        }
        Err(err) => {
            eprintln!("读取本地设备信息失败: {err}");
        }
    }
}

fn log_validation_success(payload: &OfflineLicensePayload) {
    println!(
        "离线密钥校验成功 -> 用户ID: {}, 用户名: {}, 设备ID: {}, 到期时间: {}",
        payload.user_id, payload.username, payload.device_id, payload.expires_at
    );
}

fn build_invalid_result(
    reason: impl Into<String>,
    expires_at: Option<String>,
    payload: Option<OfflineLicensePayload>,
) -> OfflineKeyValidationResult {
    let reason_text = reason.into();
    log_validation_failure(&reason_text, payload.as_ref());
    OfflineKeyValidationResult {
        is_valid: false,
        reason: Some(reason_text),
        expires_at,
        payload,
    }
}
```
pub async fn validate_from_env(lock: &Arc<Mutex<()>>) -> OfflineKeyValidationResult {
    match try_validate_from_env(lock).await {
        Ok(result) => result,
        Err(error) => {
            let reason = error.to_string();
            log_validation_failure(&reason, None);
            OfflineKeyValidationResult {
                is_valid: false,
                reason: Some(reason),
                expires_at: None,
                payload: None,
            }
        }
    }
}

async fn try_validate_from_env(
    lock: &Arc<Mutex<()>>,
) -> Result<OfflineKeyValidationResult, OfflineKeyError> {
    let env_path = match std::env::var(APP_OFFLINE_KEY_ENV_NAME) {
        Ok(value) => PathBuf::from(value),
        Err(_) => {
            return Ok(build_invalid_result(
                format!("环境变量 {APP_OFFLINE_KEY_ENV_NAME} 未设置"),
                None,
                None,
            ));
        }
    };

    if !env_path.exists() {
        return Ok(build_invalid_result("离线密钥文件不存在", None, None));
    }

    let raw = read_file_with_lock(&env_path, lock).await?;
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Ok(build_invalid_result("离线密钥文件为空", None, None));
    }

    let (rsa_part, aes_part) = split_offline_key(trimmed)?;
    let payload = decrypt_license_payload(&rsa_part)?;
    let server_time = decrypt_server_time(&aes_part)?;

    let expires_at = chrono::DateTime::parse_from_rfc3339(&payload.expires_at)
        .map_err(|_| OfflineKeyError::InvalidData("离线密钥到期时间无效".into()))?
        .with_timezone(&Utc);

    let now = Utc::now();
    if now > expires_at {
        return Ok(build_invalid_result(
            "离线密钥已过期",
            Some(payload.expires_at.clone()),
            Some(payload),
        ));
    }

    let device_id = device::get_device_id()
        .map_err(|err| OfflineKeyError::InvalidData(format!("无法获取设备标识: {err}")))?;
    if payload.device_id != device_id {
        return Ok(build_invalid_result(
            "当前设备与离线密钥绑定设备不一致",
            Some(payload.expires_at.clone()),
            Some(payload),
        ));
    }

    if now < server_time {
        return Ok(build_invalid_result(
            "检测到离线密钥时间被回滚，请重新获取",
            Some(payload.expires_at.clone()),
            Some(payload),
        ));
    }

    let updated_aes_part = encrypt_current_time(now)?;
    let updated_raw = combine_offline_key(&rsa_part, &updated_aes_part);
    write_file_with_lock(&env_path, updated_raw.as_bytes(), lock).await?;

    log_validation_success(&payload);

    Ok(OfflineKeyValidationResult {
        is_valid: true,
        reason: None,
        expires_at: Some(payload.expires_at.clone()),
        payload: Some(payload),
    })
}

async fn read_file_with_lock(
    path: &Path,
    lock: &Arc<Mutex<()>>,
) -> Result<String, OfflineKeyError> {
    let _guard = lock.lock().await;
    let content = tokio::fs::read_to_string(path).await?;
    Ok(content)
}

async fn write_file_with_lock(
    path: &Path,
    content: &[u8],
    lock: &Arc<Mutex<()>>,
) -> Result<(), OfflineKeyError> {
    let _guard = lock.lock().await;
    tokio::fs::write(path, content).await?;
    Ok(())
}

fn split_offline_key(raw: &str) -> Result<(String, String), OfflineKeyError> {
    let parts: Vec<&str> = raw.split(OFFLINE_KEY_SEPARATOR).collect();
    if parts.len() != 2 || parts.iter().any(|part| part.trim().is_empty()) {
        return Err(OfflineKeyError::InvalidFormat);
    }
    Ok((parts[0].to_string(), parts[1].to_string()))
}

fn decrypt_license_payload(encoded: &str) -> Result<OfflineLicensePayload, OfflineKeyError> {
    let private_key = &*PRIVATE_KEY;
    let encrypted = general_purpose::STANDARD
        .decode(encoded)
        .map_err(|_| OfflineKeyError::InvalidData("RSA 密文格式无效".into()))?;
    let padding = Oaep::new::<Sha256>();
    let decrypted = private_key
        .decrypt(padding, &encrypted)
        .map_err(|err| OfflineKeyError::Crypto(format!("RSA 解密失败: {err}")))?;
    let value: Value = serde_json::from_slice(&decrypted)
        .map_err(|_| OfflineKeyError::InvalidData("离线密钥数据不完整".into()))?;
    OfflineLicensePayload::from_value(&value)
}
```
fn decrypt_server_time(encoded: &str) -> Result<DateTime<Utc>, OfflineKeyError> {
    let combined = general_purpose::STANDARD
        .decode(encoded)
        .map_err(|_| OfflineKeyError::InvalidData("时间密文格式无效".into()))?;
    if combined.len() <= 12 {
        return Err(OfflineKeyError::InvalidData("离线时间数据损坏".into()));
    }
    let (nonce, ciphertext) = combined.split_at(12);
    let cipher = Aes256Gcm::new_from_slice(&AES_KEY[..])
        .map_err(|err| OfflineKeyError::Crypto(format!("AES 密钥初始化失败: {err}")))?;
    let plaintext = cipher
        .decrypt(Nonce::from_slice(nonce), ciphertext)
        .map_err(|err| OfflineKeyError::Crypto(format!("AES 解密失败: {err}")))?;
    let time_str = String::from_utf8(plaintext)
        .map_err(|_| OfflineKeyError::InvalidData("服务器时间格式无效".into()))?;
    DateTime::parse_from_rfc3339(&time_str)
        .map(|dt| dt.with_timezone(&Utc))
        .map_err(|_| OfflineKeyError::InvalidData("服务器时间格式无效".into()))
}

fn encrypt_current_time(now: DateTime<Utc>) -> Result<String, OfflineKeyError> {
    let cipher = Aes256Gcm::new_from_slice(&AES_KEY[..])
        .map_err(|err| OfflineKeyError::Crypto(format!("AES 密钥初始化失败: {err}")))?;
    let mut nonce_bytes = [0u8; 12];
    OsRng.fill_bytes(&mut nonce_bytes);
    let ciphertext = cipher
        .encrypt(Nonce::from_slice(&nonce_bytes), now.to_rfc3339().as_bytes())
        .map_err(|err| OfflineKeyError::Crypto(format!("AES 加密失败: {err}")))?;
    let mut combined = Vec::with_capacity(nonce_bytes.len() + ciphertext.len());
    combined.extend_from_slice(&nonce_bytes);
    combined.extend_from_slice(&ciphertext);
    Ok(general_purpose::STANDARD.encode(combined))
}

fn combine_offline_key(rsa_part: &str, aes_part: &str) -> String {
    format!("{rsa_part}{OFFLINE_KEY_SEPARATOR}{aes_part}")
}

#[tauri::command]
pub async fn validate_offline_key(
    lock: tauri::State<'_, FileWriteLock>,
) -> Result<OfflineKeyValidationResult, String> {
    Ok(validate_from_env(&lock.0).await)
}

#[tauri::command]
pub async fn get_device_id() -> Result<String, String> {
    device::get_device_id().map_err(|err| err.to_string())
}
```

## 设备指纹与存储辅助模块

### `src-tauri/src/device.rs`

```rust
use crate::storage;
use anyhow::{anyhow, Result};
use get_if_addrs::{get_if_addrs, IfAddr};
use machine_uid::get as get_machine_uid;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::PathBuf;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DeviceInfo {
    pub device_id: String,
    pub device_name: String,
    pub os: String,
    pub arch: String,
    pub hostname: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

pub fn get_device_id() -> Result<String> {
    if let Ok(existing_id) = load_device_id() {
        return Ok(existing_id);
    }

    let device_id = generate_device_id()?;
    save_device_id(&device_id)?;

    Ok(device_id)
}

fn generate_device_id() -> Result<String> {
    if let Ok(machine_id) = get_machine_uid() {
        let hashed = hash_identifier(machine_id.as_bytes());
        if let Some(id) = hashed {
            return Ok(id);
        }
    }

    let mut hasher = Sha256::new();

    if let Ok(hostname) = hostname::get() {
        hasher.update(hostname.to_string_lossy().as_bytes());
    }

    if let Ok(interfaces) = get_if_addrs() {
        for interface in interfaces {
            match interface.addr {
                IfAddr::V4(ifv4) => {
                    hasher.update(ifv4.ip.octets());
                    hasher.update(ifv4.netmask.octets());
                    if let Some(broadcast) = ifv4.broadcast {
                        hasher.update(broadcast.octets());
                    }
                }
                IfAddr::V6(ifv6) => {
                    hasher.update(ifv6.ip.octets());
                    hasher.update(ifv6.netmask.octets());
                    if let Some(broadcast) = ifv6.broadcast {
                        hasher.update(broadcast.octets());
                    }
                }
            }
        }
    }

    hasher.update(std::env::consts::OS.as_bytes());
    hasher.update(std::env::consts::ARCH.as_bytes());

    let fingerprint = hasher.finalize();
    let fingerprint_hex = format!("{:x}", fingerprint);

    if let Some(id) = shorten_hex(fingerprint_hex) {
        return Ok(id);
    }

    let uuid = Uuid::new_v4();
    hash_identifier(uuid.as_bytes()).ok_or_else(|| anyhow!("无法生成设备ID"))
}

fn load_device_id() -> Result<String> {
    let config_path = get_device_config_path()?;
    let content = fs::read_to_string(config_path)?;
    let device_info: DeviceInfo = serde_json::from_str(&content)?;
    Ok(device_info.device_id)
}

fn save_device_id(device_id: &str) -> Result<()> {
    let config_path = get_device_config_path()?;

    if let Some(parent) = config_path.parent() {
        fs::create_dir_all(parent)?;
    }

    let device_info = DeviceInfo {
        device_id: device_id.to_string(),
        device_name: get_device_name()?,
        os: std::env::consts::OS.to_string(),
        arch: std::env::consts::ARCH.to_string(),
        hostname: hostname::get()
            .map(|h| h.to_string_lossy().to_string())
            .unwrap_or_else(|_| "Unknown".to_string()),
        created_at: chrono::Utc::now(),
    };

    let content = serde_json::to_string_pretty(&device_info)?;
    fs::write(config_path, content)?;

    Ok(())
}

fn get_device_config_path() -> Result<PathBuf> {
    let config_dir = storage::get_app_config_dir()?;
    Ok(config_dir.join("device.json"))
}

fn get_device_name() -> Result<String> {
    if let Ok(hostname) = hostname::get() {
        return Ok(hostname.to_string_lossy().to_string());
    }

    if let Ok(name) = std::env::var("COMPUTERNAME") {
        return Ok(name);
    }

    if let Ok(name) = std::env::var("HOSTNAME") {
        return Ok(name);
    }

    Ok("Unknown Device".to_string())
}

fn hash_identifier(data: &[u8]) -> Option<String> {
    if data.is_empty() {
        return None;
    }

    let mut hasher = Sha256::new();
    hasher.update(data);
    let hex = format!("{:x}", hasher.finalize());
    shorten_hex(hex)
}

fn shorten_hex(hex: String) -> Option<String> {
    if hex.is_empty() {
        return None;
    }

    let id: String = hex.chars().take(32).collect();
    if id.is_empty() {
        None
    } else {
        Some(id)
    }
}

pub fn get_device_info() -> Result<DeviceInfo> {
    let config_path = get_device_config_path()?;

    if config_path.exists() {
        let content = fs::read_to_string(config_path)?;
        let device_info: DeviceInfo = serde_json::from_str(&content)?;
        Ok(device_info)
    } else {
        let _device_id = get_device_id()?;
        get_device_info()
    }
}
```

### `src-tauri/src/storage.rs`

```rust
use anyhow::{Context, Result};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::Mutex;

pub fn resolve_client_root() -> Result<PathBuf> {
    let mut dir = env::current_dir()?;
    for _ in 0..5 {
        if dir.join("package.json").exists() && dir.join("src-tauri").exists() {
            return Ok(dir);
        }
        if !dir.pop() {
            break;
        }
    }

    if let Ok(mut exe_dir) = env::current_exe() {
        for _ in 0..7 {
            if exe_dir.join("package.json").exists() && exe_dir.join("src-tauri").exists() {
                return Ok(exe_dir);
            }
            if !exe_dir.pop() {
                break;
            }
        }
    }

    env::current_dir().map_err(Into::into)
}

pub fn ensure_dir(path: &Path) -> Result<PathBuf> {
    if !path.exists() {
        fs::create_dir_all(path).with_context(|| format!("无法创建目录: {}", path.display()))?;
    }
    Ok(path.to_path_buf())
}

pub fn create_dir(path: &str) -> Result<PathBuf> {
    let buf = PathBuf::from(path);
    ensure_dir(&buf)
}

pub fn get_app_data_dir() -> Result<PathBuf> {
    let root = resolve_client_root()?;
    let dir = root.join("data");
    ensure_dir(&dir)
}

pub fn get_app_config_dir() -> Result<PathBuf> {
    let dir = get_app_data_dir()?.join("config");
    ensure_dir(&dir)
}

pub async fn write_file(path: &str, contents: &str, lock: &Arc<Mutex<()>>) -> Result<()> {
    let path_buf = PathBuf::from(path);
    if let Some(parent) = path_buf.parent() {
        ensure_dir(parent)?;
    }
    let _guard = lock.lock().await;
    tokio::fs::write(&path_buf, contents)
        .await
        .with_context(|| format!("写入文件失败: {}", path_buf.display()))
}

pub async fn read_file(path: &Path, lock: &Arc<Mutex<()>>) -> Result<Option<String>> {
    let _guard = lock.lock().await;
    if !path.exists() {
        return Ok(None);
    }
    let content = tokio::fs::read_to_string(path)
        .await
        .with_context(|| format!("读取文件失败: {}", path.display()))?;
    if content.trim().is_empty() {
        Ok(None)
    } else {
        Ok(Some(content))
    }
}
```
