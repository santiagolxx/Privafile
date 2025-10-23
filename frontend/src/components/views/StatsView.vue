<template>
    <div class="max-w-4xl mx-auto">
        <div class="grid grid-cols-1 md:grid-cols-3 gap-6">
            <div class="bg-white rounded-lg shadow p-6">
                <div class="flex items-center justify-between">
                    <div>
                        <p class="text-sm text-gray-600 mb-1">
                            Total de Archivos
                        </p>
                        <p class="text-3xl font-bold text-gray-900">
                            {{ fileStats.total }}
                        </p>
                    </div>
                    <FolderOpen class="w-12 h-12 text-blue-500" />
                </div>
            </div>

            <div class="bg-white rounded-lg shadow p-6">
                <div class="flex items-center justify-between">
                    <div>
                        <p class="text-sm text-gray-600 mb-1">Imágenes</p>
                        <p class="text-3xl font-bold text-gray-900">
                            {{ fileStats.images }}
                        </p>
                    </div>
                    <Image class="w-12 h-12 text-purple-500" />
                </div>
            </div>

            <div class="bg-white rounded-lg shadow p-6">
                <div class="flex items-center justify-between">
                    <div>
                        <p class="text-sm text-gray-600 mb-1">Videos</p>
                        <p class="text-3xl font-bold text-gray-900">
                            {{ fileStats.videos }}
                        </p>
                    </div>
                    <Film class="w-12 h-12 text-red-500" />
                </div>
            </div>
        </div>

        <div class="mt-6 bg-white rounded-lg shadow p-6">
            <h3 class="text-lg font-semibold text-gray-900 mb-4">
                Información del Servidor
            </h3>
            <div class="space-y-2 text-sm">
                <div class="flex justify-between">
                    <span class="text-gray-600">URL del Servidor:</span>
                    <span class="font-mono text-gray-900">
                        {{ serverUrl }}
                    </span>
                </div>
                <div class="flex justify-between">
                    <span class="text-gray-600">Estado:</span>
                    <span class="text-green-600 font-medium"> Conectado </span>
                </div>
            </div>
        </div>
    </div>
</template>

<script setup>
import { FolderOpen, Image, Film } from "lucide-vue-next";
import { useAppStore } from "@/stores/appStore";

const store = useAppStore();
const { fileStats, serverUrl } = store;
</script>
