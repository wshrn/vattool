<template>
  <div class="flex flex-col h-screen min-h-screen bg-gray-50 dark:bg-gray-950 text-gray-900 dark:text-gray-50 border border-gray-200 dark:border-gray-800">
    <TitleBar />

    <div class="flex flex-1 overflow-hidden min-h-0">
      <aside class="w-64 bg-white dark:bg-gray-900 border-r border-gray-200 dark:border-gray-800 flex flex-col px-4 py-6 space-y-6">
        <div class="space-y-2">
          <h2 class="text-sm font-semibold text-gray-700 dark:text-gray-200">主题设置</h2>
          <ThemeSwitcher />
        </div>
        <div>
          <h2 class="text-sm font-semibold text-gray-700 dark:text-gray-200 mb-2">工具简介</h2>
          <p class="text-xs leading-6 text-gray-500 dark:text-gray-400">
            本工具用于快速检测并初始化当前目录下 Python 运行环境，确保环境变量与国内源配置正确。
          </p>
        </div>
      </aside>

      <div class="flex-1 flex flex-col overflow-hidden">
        <header class="bg-white/80 dark:bg-gray-900/80 border-b border-gray-200 dark:border-gray-800 backdrop-blur-sm px-8 py-4">
          <h1 class="text-xl font-semibold">Python 环境初始化助手</h1>
          <p class="text-sm text-gray-500 dark:text-gray-400 mt-1">一键检测与配置 Python 3 环境</p>
        </header>

        <main class="flex-1 min-h-0 overflow-y-auto scrollbar-thin p-8">
          <router-view />
        </main>
      </div>
    </div>

    <NotificationContainer />
  </div>
</template>

<script setup lang="ts">
import TitleBar from '@/components/common/TitleBar.vue'
import ThemeSwitcher from '@/components/common/ThemeSwitcher.vue'
import NotificationContainer from '@/components/NotificationContainer.vue'
</script>
