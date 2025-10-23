<template>
    <div class="min-h-screen bg-gray-50" @dragover.prevent="noop">
        <!-- Global Drag Overlay -->
        <div
            v-if="globalDragging"
            class="fixed inset-0 bg-blue-500/20 backdrop-blur-sm flex items-center justify-center text-blue-700 font-bold text-2xl pointer-events-none z-50"
        >
            Suelta para subir archivo
        </div>

        <!-- Login View -->
        <div
            v-if="!isAuthenticated"
            class="flex items-center justify-center min-h-screen p-4"
        >
            <div class="w-full max-w-md">
                <div class="bg-white rounded-lg shadow-lg p-8">
                    <div class="flex items-center justify-center mb-8">
                        <HardDrive class="w-12 h-12 text-blue-600" />
                    </div>
                    <h1 class="text-2xl font-bold text-center mb-2">
                        Custom Drive
                    </h1>
                    <p class="text-gray-600 text-center mb-8">
                        Ingresa tus credenciales para continuar
                    </p>

                    <form @submit.prevent="handleLogin" class="space-y-4">
                        <div>
                            <label
                                class="block text-sm font-medium text-gray-700 mb-2"
                            >
                                URL del Servidor
                            </label>
                            <input
                                v-model="serverUrl"
                                type="text"
                                placeholder="http://localhost:5830"
                                class="w-full px-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-transparent outline-none"
                                required
                            />
                        </div>

                        <div>
                            <label
                                class="block text-sm font-medium text-gray-700 mb-2"
                            >
                                Token de Acceso
                            </label>
                            <input
                                v-model="token"
                                type="password"
                                placeholder="Ingresa tu token"
                                class="w-full px-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-transparent outline-none"
                                required
                            />
                        </div>

                        <button
                            type="submit"
                            :disabled="loading"
                            class="w-full bg-blue-600 text-white py-2 px-4 rounded-lg hover:bg-blue-700 transition-colors disabled:opacity-50 disabled:cursor-not-allowed font-medium"
                        >
                            {{ loading ? "Conectando..." : "Iniciar Sesión" }}
                        </button>
                    </form>

                    <p
                        v-if="error"
                        class="mt-4 text-sm text-red-600 text-center"
                    >
                        {{ error }}
                    </p>
                </div>
            </div>
        </div>

        <!-- Drive View with Sidebar -->
        <div v-else class="flex h-screen overflow-hidden">
            <!-- Sidebar -->
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
                        <span class="font-semibold text-gray-900"
                            >Custom Drive</span
                        >
                    </div>
                    <button
                        @click="sidebarOpen = !sidebarOpen"
                        class="p-2 hover:bg-gray-100 rounded-lg transition-colors"
                    >
                        <Menu class="w-5 h-5 text-gray-600" />
                    </button>
                </div>

                <!-- Sidebar Navigation -->
                <nav class="flex-1 p-4 space-y-2 overflow-y-auto">
                    <button
                        @click="currentView = 'files'"
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
                        @click="currentView = 'upload'"
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
                        @click="currentView = 'stats'"
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
                        @click="handleLogout"
                        class="w-full flex items-center gap-3 px-3 py-2 text-gray-700 hover:bg-gray-100 rounded-lg transition-colors"
                    >
                        <LogOut class="w-5 h-5 flex-shrink-0" />
                        <span v-show="sidebarOpen">Cerrar Sesión</span>
                    </button>
                </div>
            </aside>

            <!-- Main Content -->
            <div class="flex-1 flex flex-col overflow-hidden">
                <header class="bg-white border-b border-gray-200 px-6 py-4">
                    <div class="flex items-center justify-between">
                        <h1 class="text-xl font-bold text-gray-900">
                            {{ getViewTitle() }}
                        </h1>

                        <button
                            @click="fetchFiles"
                            :disabled="loading"
                            class="p-2 text-gray-600 hover:text-gray-900 hover:bg-gray-100 rounded-lg transition-colors"
                            title="Actualizar"
                        >
                            <RefreshCw
                                :class="['w-5 h-5', loading && 'animate-spin']"
                            />
                        </button>
                    </div>
                </header>

                <!-- Content -->
                <main class="flex-1 overflow-y-auto p-6">
                    <!-- Upload View -->
                    <div
                        v-if="currentView === 'upload'"
                        class="max-w-4xl mx-auto"
                    >
                        <div
                            @dragover.prevent="isDragging = true"
                            @dragleave.prevent="isDragging = false"
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
                                    isDragging
                                        ? 'text-blue-500'
                                        : 'text-gray-400',
                                ]"
                            />
                            <h3
                                class="text-lg font-semibold text-gray-900 mb-2"
                            >
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

                        <div
                            v-if="selectedFile"
                            class="mt-6 bg-white rounded-lg shadow p-6"
                        >
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
                                            {{
                                                formatFileSize(
                                                    selectedFile.size,
                                                )
                                            }}
                                        </p>
                                    </div>
                                </div>

                                <div class="flex items-center gap-2">
                                    <button
                                        @click="uploadFile"
                                        :disabled="uploading"
                                        class="px-4 py-2 bg-green-600 text-white rounded-lg hover:bg-green-700 transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
                                    >
                                        {{
                                            uploading
                                                ? "Subiendo..."
                                                : "Subir Archivo"
                                        }}
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
                                <div
                                    class="w-full bg-gray-200 rounded-full h-2"
                                >
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
                                    uploadSuccess
                                        ? 'text-green-600'
                                        : 'text-red-600',
                                ]"
                            >
                                {{ uploadMessage }}
                            </p>
                        </div>
                    </div>

                    <!-- Files View -->
                    <div v-else-if="currentView === 'files'">
                        <div
                            v-if="loading && files.length === 0"
                            class="flex flex-col items-center justify-center py-20"
                        >
                            <RefreshCw
                                class="w-12 h-12 text-gray-400 animate-spin mb-4"
                            />
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
                                @click="currentView = 'upload'"
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
                            >
                                <div
                                    class="flex flex-col items-center text-center"
                                >
                                    <component
                                        :is="getFileIcon(file.mime)"
                                        class="w-16 h-16 mb-4 transition-transform group-hover:scale-110"
                                        :class="getFileColor(file.mime)"
                                    />

                                    <p
                                        class="text-sm font-medium text-gray-900 mb-2 break-all"
                                    >
                                        {{ getFileName(file) }}
                                    </p>

                                    <span
                                        class="inline-block px-2 py-1 text-xs font-medium rounded-full mb-2"
                                        :class="
                                            getMimeTypeBadgeClass(file.mime)
                                        "
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
                    </div>

                    <!-- Stats View -->
                    <div
                        v-else-if="currentView === 'stats'"
                        class="max-w-4xl mx-auto"
                    >
                        <div class="grid grid-cols-1 md:grid-cols-3 gap-6">
                            <div class="bg-white rounded-lg shadow p-6">
                                <div class="flex items-center justify-between">
                                    <div>
                                        <p class="text-sm text-gray-600 mb-1">
                                            Total de Archivos
                                        </p>
                                        <p
                                            class="text-3xl font-bold text-gray-900"
                                        >
                                            {{ files.length }}
                                        </p>
                                    </div>
                                    <FolderOpen
                                        class="w-12 h-12 text-blue-500"
                                    />
                                </div>
                            </div>

                            <div class="bg-white rounded-lg shadow p-6">
                                <div class="flex items-center justify-between">
                                    <div>
                                        <p class="text-sm text-gray-600 mb-1">
                                            Imágenes
                                        </p>
                                        <p
                                            class="text-3xl font-bold text-gray-900"
                                        >
                                            {{ getFileTypeCount("image") }}
                                        </p>
                                    </div>
                                    <Image class="w-12 h-12 text-purple-500" />
                                </div>
                            </div>

                            <div class="bg-white rounded-lg shadow p-6">
                                <div class="flex items-center justify-between">
                                    <div>
                                        <p class="text-sm text-gray-600 mb-1">
                                            Videos
                                        </p>
                                        <p
                                            class="text-3xl font-bold text-gray-900"
                                        >
                                            {{ getFileTypeCount("video") }}
                                        </p>
                                    </div>
                                    <Film class="w-12 h-12 text-red-500" />
                                </div>
                            </div>
                        </div>

                        <div class="mt-6 bg-white rounded-lg shadow p-6">
                            <h3
                                class="text-lg font-semibold text-gray-900 mb-4"
                            >
                                Información del Servidor
                            </h3>
                            <div class="space-y-2 text-sm">
                                <div class="flex justify-between">
                                    <span class="text-gray-600"
                                        >URL del Servidor:</span
                                    >
                                    <span class="font-mono text-gray-900">
                                        {{ serverUrl }}
                                    </span>
                                </div>
                                <div class="flex justify-between">
                                    <span class="text-gray-600">Estado:</span>
                                    <span class="text-green-600 font-medium">
                                        Conectado
                                    </span>
                                </div>
                            </div>
                        </div>
                    </div>
                </main>
            </div>
        </div>
    </div>
</template>

<script setup>
import { ref, onMounted, onUnmounted } from "vue";
import {
    HardDrive,
    LogOut,
    RefreshCw,
    Upload,
    FolderOpen,
    FileText,
    Image,
    Film,
    Music,
    File,
    Menu,
    BarChart3,
    X,
} from "lucide-vue-next";

const isAuthenticated = ref(false);
const serverUrl = ref("http://localhost:5830");
const token = ref("");
const loading = ref(false);
const error = ref("");

const files = ref([]);
const selectedFile = ref(null);
const uploading = ref(false);
const uploadMessage = ref("");
const uploadSuccess = ref(false);
const fileInput = ref(null);

const sidebarOpen = ref(true);
const currentView = ref("files");

const isDragging = ref(false);
const globalDragging = ref(false);

const noop = () => {}; // evita el evento nativo por defecto

// Intento autologin
onMounted(() => {
    const stored = localStorage.getItem("token");
    if (stored) {
        token.value = stored;
        fetchFiles()
            .then(() => (isAuthenticated.value = true))
            .catch(() => {
                localStorage.removeItem("token");
            });
    }
});

const handleLogin = async () => {
    loading.value = true;
    error.value = "";

    try {
        await fetchFiles();
        isAuthenticated.value = true;
        localStorage.setItem("token", token.value);
    } catch (err) {
        error.value =
            "Error al conectar con el servidor. Verifica tus credenciales.";
    } finally {
        loading.value = false;
    }
};

const fetchFiles = async () => {
    loading.value = true;
    error.value = "";

    try {
        const response = await fetch(`${serverUrl.value}/api/files/list`, {
            headers: {
                Authorization: `Bearer ${token.value}`,
            },
        });

        const data = await response.json();

        if (data.success) {
            files.value = data.files || [];
        } else {
            throw new Error(data.message || "Error al obtener archivos");
        }
    } catch (err) {
        error.value = err.message;
        if (!isAuthenticated.value) {
            throw err;
        }
    } finally {
        loading.value = false;
    }
};

const handleFileSelect = (event) => {
    const file = event.target.files[0];
    if (file) {
        selectedFile.value = file;
        uploadMessage.value = "";
    }
};

const handleDrop = (event) => {
    isDragging.value = false;
    const file = event.dataTransfer.files[0];
    if (file) {
        selectedFile.value = file;
        uploadMessage.value = "";
    }
};

const clearSelectedFile = () => {
    selectedFile.value = null;
    uploadMessage.value = "";
    if (fileInput.value) {
        fileInput.value.value = "";
    }
};

const uploadFile = async () => {
    if (!selectedFile.value) return;

    uploading.value = true;
    uploadMessage.value = "";

    try {
        const mimeType = encodeURIComponent(
            selectedFile.value.type || "application/octet-stream",
        );

        const response = await fetch(
            `${serverUrl.value}/api/files/upload?mime=${mimeType}`,
            {
                method: "POST",
                headers: {
                    Authorization: `Bearer ${token.value}`,
                    "Content-Type":
                        selectedFile.value.type || "application/octet-stream",
                },
                body: selectedFile.value,
            },
        );

        const data = await response.json();

        if (data.success) {
            uploadMessage.value = data.message || "Archivo subido exitosamente";
            uploadSuccess.value = true;

            await fetchFiles();

            setTimeout(() => {
                clearSelectedFile();
            }, 2000);
        } else {
            throw new Error(data.message || "Error al subir archivo");
        }
    } catch (err) {
        uploadMessage.value = err.message;
        uploadSuccess.value = false;
    } finally {
        uploading.value = false;
    }
};

const handleLogout = () => {
    isAuthenticated.value = false;
    files.value = [];
    token.value = "";
    error.value = "";
    currentView.value = "files";
    localStorage.removeItem("token");
};

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

const formatFileSize = (bytes) => {
    if (bytes < 1024) return bytes + " B";
    if (bytes < 1024 * 1024) return (bytes / 1024).toFixed(2) + " KB";
    return (bytes / (1024 * 1024)).toFixed(2) + " MB";
};

const getViewTitle = () => {
    if (currentView.value === "files") return "Mis Archivos";
    if (currentView.value === "upload") return "Subir Archivos";
    if (currentView.value === "stats") return "Estadísticas";
    return "Custom Drive";
};

const getFileTypeCount = (type) => {
    return files.value.filter((file) => file.mime.startsWith(type)).length;
};

/* --- GLOBAL DRAG & DROP --- */
onMounted(() => {
    const enter = () => (globalDragging.value = true);
    const leave = () => (globalDragging.value = false);

    window.addEventListener("dragover", enter);
    window.addEventListener("dragleave", leave);

    window.addEventListener("drop", (e) => {
        e.preventDefault();
        globalDragging.value = false;

        const file = e.dataTransfer.files[0];
        if (file) {
            selectedFile.value = file;
            currentView.value = "upload";
        }
    });
});

onUnmounted(() => {
    window.removeEventListener("dragover", () => {});
    window.removeEventListener("dragleave", () => {});
    window.removeEventListener("drop", () => {});
});
</script>
