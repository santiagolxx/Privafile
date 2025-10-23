<template>
    <div>
        <div
            v-if="loading && files.length === 0"
            class="flex flex-col items-center justify-center py-20"
        >
            <RefreshCw class="w-12 h-12 text-gray-400 animate-spin mb-4" />
            <p class="text-gray-600">Cargando archivos...</p>
        </div>

        <div
            v-else-if="files.length === 0"
            class="flex flex-col items-center justify-center py-20"
        >
            <FolderOpen class="w-20 h-20 text-gray-300 mb-4" />
            <p class="text-gray-600 text-lg mb-2">
                No hay archivos disponibles
            </p>
            <button
                @click="setView('upload')"
                class="mt-4 px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors"
            >
                Subir tu primer archivo
            </button>
        </div>

        <div
            v-else
            class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-4"
        >
            <div
                v-for="file in files"
                :key="file.id"
                class="bg-white border border-gray-200 rounded-lg p-6 hover:shadow-lg transition-all cursor-pointer group"
                @click="handleFileClick(file)"
            >
                <div class="flex flex-col items-center text-center">
                    <component
                        :is="getFileIcon(file.mime)"
                        class="w-16 h-16 mb-4 transition-transform group-hover:scale-110"
                        :class="getFileColor(file.mime)"
                    />

                    <p class="text-sm font-medium text-gray-900 mb-2 break-all">
                        {{ getFileName(file) }}
                    </p>

                    <span
                        class="inline-block px-2 py-1 text-xs font-medium rounded-full mb-2"
                        :class="getMimeTypeBadgeClass(file.mime)"
                    >
                        {{ getMimeTypeLabel(file.mime) }}
                    </span>

                    <p
                        class="text-xs text-gray-400 font-mono truncate w-full"
                        :title="file.hash"
                    >
                        {{ file.hash.substring(0, 16) }}...
                    </p>
                </div>
            </div>
        </div>

        <!-- Modal para video -->
        <VideoModal
            v-if="selectedVideo"
            :show="!!selectedVideo"
            :video-url="selectedVideoUrl"
            :title="selectedVideoTitle"
            @close="selectedVideo = null"
        />
    </div>
</template>

<script setup>
import { ref, computed } from "vue";
import {
    FolderOpen,
    RefreshCw,
    Image,
    Film,
    Music,
    FileText,
    File,
} from "lucide-vue-next";
import { useAppStore } from "@/stores/appStore";
import { driveAPI } from "@/lib/privalib";
import VideoModal from "@/components/player/VideoModal.vue";

const store = useAppStore();
const { files, loading, setView } = store;

const selectedVideo = ref(null);

// Corregir: Usar los métodos estáticos correctamente desde DriveAPI
const getFileIcon = (mime) => {
    if (mime.startsWith("image/")) return Image;
    if (mime.startsWith("video/")) return Film;
    if (mime.startsWith("audio/")) return Music;
    if (mime.includes("pdf") || mime.includes("document")) return FileText;
    return File;
};

const getFileColor = (mime) => {
    if (mime.startsWith("image/")) return "text-purple-500";
    if (mime.startsWith("video/")) return "text-red-500";
    if (mime.startsWith("audio/")) return "text-green-500";
    if (mime.includes("pdf")) return "text-red-600";
    return "text-gray-500";
};

const getMimeTypeLabel = (mime) => {
    const parts = mime.split("/");
    return parts[parts.length - 1].toUpperCase();
};

const getMimeTypeBadgeClass = (mime) => {
    if (mime.startsWith("image/")) return "bg-purple-100 text-purple-700";
    if (mime.startsWith("video/")) return "bg-red-100 text-red-700";
    if (mime.startsWith("audio/")) return "bg-green-100 text-green-700";
    if (mime.includes("pdf")) return "bg-orange-100 text-orange-700";
    return "bg-gray-100 text-gray-700";
};

const getFileName = (file) => {
    return `${file.id.substring(0, 8)}...`;
};

const selectedVideoUrl = computed(() => {
    if (!selectedVideo.value) return "";
    return driveAPI.getDownloadUrl(selectedVideo.value);
});

const selectedVideoTitle = computed(() => {
    if (!selectedVideo.value) return "";
    return getFileName(selectedVideo.value);
});

const handleFileClick = (file) => {
    if (file.mime.startsWith("video/")) {
        selectedVideo.value = file;
    }
    // Para otros tipos de archivos, podrías manejarlos de otra manera
};
</script>
