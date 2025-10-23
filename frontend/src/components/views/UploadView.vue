<template>
    <div class="max-w-4xl mx-auto">
        <div
            @dragover.prevent="setDragging(true)"
            @dragleave.prevent="setDragging(false)"
            @drop.prevent="handleDrop"
            :class="[
                'border-2 border-dashed rounded-lg p-12 text-center transition-all',
                isDragging
                    ? 'border-blue-500 bg-blue-50'
                    : 'border-gray-300 bg-white hover:border-gray-400',
            ]"
        >
            <Upload
                :class="[
                    'w-16 h-16 mx-auto mb-4',
                    isDragging ? 'text-blue-500' : 'text-gray-400',
                ]"
            />
            <h3 class="text-lg font-semibold text-gray-900 mb-2">
                Arrastra y suelta tu archivo aquí
            </h3>
            <p class="text-gray-600 mb-6">
                o haz clic en el botón para seleccionar
            </p>

            <input
                ref="fileInput"
                type="file"
                @change="handleFileSelect"
                class="hidden"
            />
            <button
                @click="$refs.fileInput.click()"
                :disabled="uploading"
                class="px-6 py-3 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors disabled:opacity-50 disabled:cursor-not-allowed font-medium"
            >
                Seleccionar Archivo
            </button>
        </div>

        <div v-if="selectedFile" class="mt-6 bg-white rounded-lg shadow p-6">
            <div class="flex items-center justify-between">
                <div class="flex items-center gap-4">
                    <component
                        :is="getFileIcon(selectedFile.type)"
                        class="w-10 h-10 text-blue-600"
                    />
                    <div>
                        <p class="font-medium text-gray-900">
                            {{ selectedFile.name }}
                        </p>
                        <p class="text-sm text-gray-500">
                            {{ formatFileSize(selectedFile.size) }}
                        </p>
                    </div>
                </div>

                <div class="flex items-center gap-2">
                    <button
                        @click="uploadFile"
                        :disabled="uploading"
                        class="px-4 py-2 bg-green-600 text-white rounded-lg hover:bg-green-700 transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
                    >
                        {{ uploading ? "Subiendo..." : "Subir Archivo" }}
                    </button>
                    <button
                        @click="clearSelectedFile"
                        :disabled="uploading"
                        class="p-2 text-gray-600 hover:bg-gray-100 rounded-lg transition-colors"
                    >
                        <X class="w-5 h-5" />
                    </button>
                </div>
            </div>

            <div v-if="uploading" class="mt-4">
                <div class="w-full bg-gray-200 rounded-full h-2">
                    <div
                        class="bg-blue-600 h-2 rounded-full transition-all duration-300"
                        style="width: 100%"
                    ></div>
                </div>
            </div>

            <p
                v-if="uploadMessage"
                :class="[
                    'mt-4 text-sm',
                    uploadSuccess ? 'text-green-600' : 'text-red-600',
                ]"
            >
                {{ uploadMessage }}
            </p>
        </div>
    </div>
</template>

<script setup>
import { ref } from "vue";
import { Upload, X } from "lucide-vue-next";
import { useAppStore } from "@/stores/appStore";
import { driveAPI } from "@/lib/privalib";

const store = useAppStore();
const {
    isDragging,
    selectedFile,
    uploading,
    uploadMessage,
    uploadSuccess,
    setDragging,
    setSelectedFile,
    clearSelectedFile,
    uploadFile: uploadFileAction,
} = store;

const fileInput = ref(null);

const { getFileIcon, formatFileSize } = driveAPI;

const handleFileSelect = (event) => {
    const file = event.target.files[0];
    if (file) {
        setSelectedFile(file);
    }
};

const handleDrop = (event) => {
    setDragging(false);
    const file = event.dataTransfer.files[0];
    if (file) {
        setSelectedFile(file);
    }
};

const uploadFile = () => {
    if (selectedFile) {
        uploadFileAction(selectedFile);
    }
};
</script>
