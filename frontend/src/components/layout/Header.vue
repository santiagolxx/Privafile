<template>
    <header class="bg-white border-b border-gray-200 px-6 py-4">
        <div class="flex items-center justify-between">
            <h1 class="text-xl font-bold text-gray-900">
                {{ viewTitle }}
            </h1>

            <button
                @click="fetchFiles"
                :disabled="loading"
                class="p-2 text-gray-600 hover:text-gray-900 hover:bg-gray-100 rounded-lg transition-colors"
                title="Actualizar"
            >
                <RefreshCw :class="['w-5 h-5', loading && 'animate-spin']" />
            </button>
        </div>
    </header>
</template>

<script setup>
import { computed } from "vue";
import { RefreshCw } from "lucide-vue-next";
import { useAppStore } from "@/stores/appStore";

const store = useAppStore();
const { currentView, loading, fetchFiles } = store;

const viewTitle = computed(() => {
    if (currentView === "files") return "Mis Archivos";
    if (currentView === "upload") return "Subir Archivos";
    if (currentView === "stats") return "Estadísticas";
    return "Custom Drive";
});
</script>
