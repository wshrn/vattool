<template>
  <div
    ref="rootRef"
    class="mirror-select__control"
    :class="{
      'mirror-select__control--open': isOpen,
      'mirror-select__control--disabled': disabled,
    }"
  >
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
      ref="triggerRef"
      type="button"
      class="mirror-select__input"
      :disabled="disabled"
      role="combobox"
      aria-haspopup="listbox"
      :aria-controls="dropdownId"
      :aria-expanded="isOpen"
      :aria-activedescendant="activeDescendant"
      @click="toggleDropdown"
      @keydown="handleTriggerKeydown"
    >
      <span class="mirror-select__value">
        <span class="mirror-select__value-label">{{ selectedOption?.label ?? '未选择镜像' }}</span>
        <span v-if="selectedOption" class="mirror-select__value-url">{{ selectedOption.value }}</span>
      </span>
    </button>

    <span class="mirror-select__chevron" aria-hidden="true">
      <svg viewBox="0 0 20 20" fill="none" stroke="currentColor" stroke-width="1.5">
        <path d="M6 8l4 4 4-4" stroke-linecap="round" stroke-linejoin="round" />
      </svg>
    </span>

    <span class="mirror-select__shine" aria-hidden="true" />
    <span class="mirror-select__border" aria-hidden="true" />

    <Transition name="mirror-dropdown">
      <ul
        v-if="isOpen"
        ref="menuRef"
        :id="dropdownId"
        class="mirror-select__menu"
        role="listbox"
        :aria-activedescendant="activeDescendant"
        tabindex="-1"
        @keydown="handleMenuKeydown"
      >
        <li
          v-for="(option, index) in options"
          :id="optionId(index)"
          :key="option.value"
          class="mirror-select__option"
          :class="{
            'mirror-select__option--active': index === highlightIndex,
            'mirror-select__option--selected': option.value === modelValue,
          }"
          role="option"
          :aria-selected="option.value === modelValue"
          @click="selectOption(index)"
          @mouseenter="highlightIndex = index"
        >
          <div class="mirror-select__option-label">{{ option.label }}</div>
          <div class="mirror-select__option-value">{{ option.value }}</div>
        </li>
      </ul>
    </Transition>
  </div>
</template>

<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, onMounted, ref, watch } from 'vue'

interface MirrorOption {
  label: string
  value: string
  trustedHosts?: string[]
}

const props = defineProps<{
  modelValue: string
  options: MirrorOption[]
  disabled?: boolean
}>()

const emit = defineEmits<{
  (e: 'update:modelValue', value: string): void
}>()

const rootRef = ref<HTMLElement | null>(null)
const triggerRef = ref<HTMLButtonElement | null>(null)
const menuRef = ref<HTMLUListElement | null>(null)
const isOpen = ref(false)
const highlightIndex = ref(-1)
const dropdownId = `mirror-menu-${Math.random().toString(36).slice(2, 8)}`

const optionId = (index: number) => `${dropdownId}-option-${index}`

const selectedIndex = computed(() => props.options.findIndex((option) => option.value === props.modelValue))
const selectedOption = computed(() => props.options[selectedIndex.value] ?? props.options[0])

const activeDescendant = computed(() => {
  const index = highlightIndex.value !== -1 ? highlightIndex.value : selectedIndex.value
  return index >= 0 ? optionId(index) : undefined
})

const closeDropdown = () => {
  if (!isOpen.value) return
  isOpen.value = false
  highlightIndex.value = -1
}

const scrollHighlightedIntoView = () => {
  if (highlightIndex.value === -1) return
  const menuEl = menuRef.value
  const optionEl = menuEl?.querySelector<HTMLElement>(`#${optionId(highlightIndex.value)}`)
  if (!menuEl || !optionEl) return

  const optionTop = optionEl.offsetTop
  const optionBottom = optionTop + optionEl.offsetHeight
  const viewTop = menuEl.scrollTop
  const viewBottom = viewTop + menuEl.clientHeight

  if (optionTop < viewTop) {
    menuEl.scrollTop = optionTop
  } else if (optionBottom > viewBottom) {
    menuEl.scrollTop = optionBottom - menuEl.clientHeight
  }
}

const openDropdown = () => {
  if (props.disabled || props.options.length === 0) return
  isOpen.value = true
  highlightIndex.value = selectedIndex.value !== -1 ? selectedIndex.value : 0
  nextTick(() => {
    menuRef.value?.focus()
    scrollHighlightedIntoView()
  })
}

const toggleDropdown = () => {
  if (props.disabled) return
  if (isOpen.value) {
    closeDropdown()
  } else {
    openDropdown()
  }
}

const moveHighlight = (delta: number) => {
  if (props.options.length === 0) return
  const total = props.options.length
  const start = highlightIndex.value !== -1 ? highlightIndex.value : selectedIndex.value
  const base = start !== -1 ? start : 0
  let next = (base + delta) % total
  if (next < 0) {
    next += total
  }
  highlightIndex.value = next
  nextTick(scrollHighlightedIntoView)
}

const selectOption = (index: number) => {
  const option = props.options[index]
  if (!option) return
  emit('update:modelValue', option.value)
  highlightIndex.value = index
  closeDropdown()
  nextTick(() => triggerRef.value?.focus())
}

const handleTriggerKeydown = (event: KeyboardEvent) => {
  if (props.disabled) return
  switch (event.key) {
    case 'ArrowDown':
      event.preventDefault()
      if (!isOpen.value) {
        openDropdown()
      } else {
        moveHighlight(1)
      }
      break
    case 'ArrowUp':
      event.preventDefault()
      if (!isOpen.value) {
        openDropdown()
      } else {
        moveHighlight(-1)
      }
      break
    case 'Enter':
    case ' ': {
      event.preventDefault()
      if (isOpen.value) {
        if (highlightIndex.value !== -1) {
          selectOption(highlightIndex.value)
        } else {
          closeDropdown()
        }
      } else {
        openDropdown()
      }
      break
    }
    case 'Escape':
      if (isOpen.value) {
        event.preventDefault()
        closeDropdown()
      }
      break
    default:
      break
  }
}

const handleMenuKeydown = (event: KeyboardEvent) => {
  switch (event.key) {
    case 'ArrowDown':
      event.preventDefault()
      moveHighlight(1)
      break
    case 'ArrowUp':
      event.preventDefault()
      moveHighlight(-1)
      break
    case 'Enter':
    case ' ': {
      event.preventDefault()
      if (highlightIndex.value !== -1) {
        selectOption(highlightIndex.value)
      }
      break
    }
    case 'Escape':
      event.preventDefault()
      closeDropdown()
      nextTick(() => triggerRef.value?.focus())
      break
    case 'Tab':
      closeDropdown()
      break
    default:
      break
  }
}

const handleDocumentClick = (event: MouseEvent) => {
  if (!rootRef.value) return
  if (!rootRef.value.contains(event.target as Node)) {
    closeDropdown()
  }
}

onMounted(() => {
  document.addEventListener('click', handleDocumentClick)
})

onBeforeUnmount(() => {
  document.removeEventListener('click', handleDocumentClick)
})

watch(
  () => props.modelValue,
  () => {
    if (!isOpen.value) {
      highlightIndex.value = -1
    }
  }
)

watch(
  () => props.disabled,
  (disabled) => {
    if (disabled) {
      closeDropdown()
    }
  }
)

watch(
  () => props.options,
  (options) => {
    if (options.length === 0) {
      closeDropdown()
      return
    }
    if (isOpen.value) {
      const index = selectedIndex.value !== -1 ? selectedIndex.value : 0
      highlightIndex.value = index
      nextTick(scrollHighlightedIntoView)
    }
  }
)
</script>
