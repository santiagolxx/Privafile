<template>
    <aside
        :class="[
            'bg-white border-r border-gray-200 transition-all duration-300 flex flex-col',
            sidebarOpen ? 'w-64' : 'w-0 md:w-16',
        ]"
    >
        <div
            class="p-4 border-b border-gray-200 flex items-center justify-between"
        >
            <div v-show="sidebarOpen" class="flex items-center gap-2">
                <HardDrive class="w-6 h-6 text-blue-600" />
                <span class="font-semibold text-gray-900">Custom Drive</span>
            </div>
            <button
                @click="toggleSidebar"
                class="p-2 hover:bg-gray-100 rounded-lg transition-colors"
            >
                <Menu class="w-5 h-5 text-gray-600" />
            </button>
        </div>

        <!-- Sidebar Navigation -->
        <nav class="flex-1 p-4 space-y-2 overflow-y-auto">
            <button
                @click="setView('files')"
                :class="[
                    'w-full flex items-center gap-3 px-3 py-2 rounded-lg transition-colors',
                    currentView === 'files'
                        ? 'bg-blue-50 text-blue-600'
                        : 'text-gray-700 hover:bg-gray-100',
                ]"
            >
                <FolderOpen class="w-5 h-5 flex-shrink-0" />
                <span v-show="sidebarOpen">Mis Archivos</span>
            </button>

            <button
                @click="setView('upload')"
                :class="[
                    'w-full flex items-center gap-3 px-3 py-2 rounded-lg transition-colors',
                    currentView === 'upload'
                        ? 'bg-blue-50 text-blue-600'
                        : 'text-gray-700 hover:bg-gray-100',
                ]"
            >
                <Upload class="w-5 h-5 flex-shrink-0" />
                <span v-show="sidebarOpen">Subir Archivos</span>
            </button>

            <button
                @click="setView('stats')"
                :class="[
                    'w-full flex items-center gap-3 px-3 py-2 rounded-lg transition-colors',
                    currentView === 'stats'
                        ? 'bg-blue-50 text-blue-600'
                        : 'text-gray-700 hover:bg-gray-100',
                ]"
            >
                <BarChart3 class="w-5 h-5 flex-shrink-0" />
                <span v-show="sidebarOpen">Estadísticas</span>
            </button>
        </nav>

        <div class="p-4 border-t border-gray-200">
            <button
                @click="logout"
                class="w-full flex items-center gap-3 px-3 py-2 text-gray-700 hover:bg-gray-100 rounded-lg transition-colors"
            >
                <LogOut class="w-5 h-5 flex-shrink-0" />
                <span v-show="sidebarOpen">Cerrar Sesión</span>
            </button>
        </div>
    </aside>
</template>

<script setup>
import {
    HardDrive,
    Menu,
    FolderOpen,
    Upload,
    BarChart3,
    LogOut,
} from "lucide-vue-next";
import { useAppStore } from "@/stores/appStore";

const store = useAppStore();
const { sidebarOpen, currentView, toggleSidebar, setView, logout } = store;
</script>
