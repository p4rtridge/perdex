<script setup lang="ts">
import { onMounted, onUnmounted, ref, watch } from 'vue'
import { useData } from 'vitepress'

const props = defineProps<{ code: string }>()

const { isDark } = useData()
const svgHtml = ref('')
const error = ref('')
const isOpen = ref(false)
const scale = ref(1)
const translateX = ref(0)
const translateY = ref(0)
const isDragging = ref(false)
let startX = 0
let startY = 0

const render = async () => {
  try {
    const mod = await import('mermaid')
    const mermaid = mod.default ?? mod as any

    mermaid.initialize({
      startOnLoad: false,
      theme: isDark.value ? 'dark' : 'neutral',
      sequence: { useMaxWidth: true },
    })

    const id = `mermaid-${Math.random().toString(36).slice(2, 10)}`
    const code = decodeURIComponent(props.code)

    const result = await mermaid.render(id, code)

    // Mermaid SVGs often lack explicit width/height — inject from viewBox so
    // the element has intrinsic dimensions inside a flex container.
    let svg = result.svg
    const vbMatch = svg.match(/viewBox="[^"]*?\s+([\d.]+)\s+([\d.]+)"/i)
    if (vbMatch) {
      svg = svg.replace('<svg', `<svg width="${vbMatch[1]}" height="${vbMatch[2]}"`)
    }
    svgHtml.value = svg
    error.value = ''
  } catch (e) {
    const msg = e instanceof Error ? e.message : String(e)
    console.error('[Mermaid render error]', msg, '\nCode:', decodeURIComponent(props.code))
    error.value = msg
    svgHtml.value = ''
  }
}

const onWheel = (e: WheelEvent) => {
  scale.value = Math.min(5, Math.max(0.4, scale.value - e.deltaY * 0.0015))
}

const onPointerDown = (e: PointerEvent) => {
  if (e.button !== 0) return
  isDragging.value = true
  startX = e.clientX - translateX.value
  startY = e.clientY - translateY.value
  ;(e.currentTarget as HTMLElement).setPointerCapture(e.pointerId)
}

const onPointerMove = (e: PointerEvent) => {
  if (!isDragging.value) return
  translateX.value = e.clientX - startX
  translateY.value = e.clientY - startY
}

const onPointerUp = (e: PointerEvent) => {
  isDragging.value = false
  ;(e.currentTarget as HTMLElement).releasePointerCapture(e.pointerId)
}

const onKeydown = (e: KeyboardEvent) => {
  if (e.key === 'Escape') isOpen.value = false
}

onMounted(() => {
  render()
  window.addEventListener('keydown', onKeydown)
})

onUnmounted(() => {
  window.removeEventListener('keydown', onKeydown)
})

watch(isDark, render)

watch(isOpen, (v) => {
  if (!v) {
    scale.value = 1
    translateX.value = 0
    translateY.value = 0
  }
  document.body.style.overflow = v ? 'hidden' : ''
})
</script>

<template>
  <div class="mermaid-wrapper">
    <!-- Error state -->
    <div v-if="error" class="mermaid-error">
      <strong>Mermaid error:</strong> {{ error }}
      <pre>{{ decodeURIComponent(code) }}</pre>
    </div>

    <!-- Loading / rendered state -->
    <div
      v-else
      class="mermaid-diagram"
      :class="{ 'is-empty': !svgHtml }"
      v-html="svgHtml || '<span class=\'mermaid-loading\'>Rendering…</span>'"
      :title="svgHtml ? 'Click to zoom' : undefined"
      @click="svgHtml && (isOpen = true)"
    />

    <Teleport to="body">
      <Transition name="mermaid-fade">
        <div v-if="isOpen" class="mermaid-overlay" @click.self="isOpen = false">
          <button class="mermaid-close" @click="isOpen = false">✕</button>

          <div class="mermaid-lightbox" @wheel.prevent="onWheel">
            <div
              class="mermaid-lightbox-inner"
              :style="{
                transform: `translate(${translateX}px, ${translateY}px) scale(${scale})`,
                cursor: isDragging ? 'grabbing' : 'grab',
                transition: isDragging ? 'none' : 'transform 0.08s ease-out'
              }"
              @pointerdown.prevent="onPointerDown"
              @pointermove="onPointerMove"
              @pointerup="onPointerUp"
              @pointercancel="onPointerUp"
              v-html="svgHtml"
            />
          </div>

          <p class="mermaid-hint">Scroll to zoom · Click outside or Esc to close</p>
        </div>
      </Transition>
    </Teleport>
  </div>
</template>

<style scoped>
.mermaid-diagram {
  padding: 0.5rem 0;
  display: flex;
  justify-content: center;
  align-items: center;
  cursor: zoom-in;
  border-radius: 8px;
  min-height: 3rem;
  width: 100%;
  box-sizing: border-box;
}
.mermaid-diagram.is-empty {
  cursor: default;
}
.mermaid-diagram :deep(svg) {
  max-width: 100% !important;
  height: auto !important;
}
:deep(.mermaid-loading) {
  color: var(--vp-c-text-3);
  font-size: 0.85rem;
}

.mermaid-error {
  padding: 0.75rem 1rem;
  border-radius: 6px;
  background: #fff0f0;
  color: #c00;
  font-size: 0.82rem;
  border: 1px solid #fcc;
}
.mermaid-error pre {
  margin-top: 0.5rem;
  font-size: 0.75rem;
  white-space: pre-wrap;
  word-break: break-all;
}

/* ── Overlay ─────────────────────────── */
.mermaid-overlay {
  position: fixed;
  inset: 0;
  z-index: 9999;
  background: rgba(0, 0, 0, 0.82);
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 0.75rem;
}
.mermaid-close {
  position: absolute;
  top: 1.25rem;
  right: 1.5rem;
  width: 2.1rem;
  height: 2.1rem;
  border: 1px solid rgba(255, 255, 255, 0.28);
  border-radius: 50%;
  background: transparent;
  color: #fff;
  font-size: 1rem;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: background 0.15s;
}
.mermaid-close:hover { background: rgba(255, 255, 255, 0.15); }

.mermaid-lightbox {
  overflow: hidden;
  display: flex;
  align-items: center;
  justify-content: center;
  width: 90vw;
  height: 84vh;
}
.mermaid-lightbox-inner {
  transform-origin: center center;
  user-select: none;
  touch-action: none;
}
.mermaid-lightbox-inner :deep(svg) {
  display: block;
  width: auto;
  height: auto;
  max-width: 85vw;
  max-height: 78vh;
  pointer-events: none;
}

.mermaid-hint {
  color: rgba(255, 255, 255, 0.4);
  font-size: 0.76rem;
  letter-spacing: 0.03em;
  margin: 0;
}

.mermaid-fade-enter-active,
.mermaid-fade-leave-active { transition: opacity 0.18s ease; }
.mermaid-fade-enter-from,
.mermaid-fade-leave-to { opacity: 0; }
</style>
