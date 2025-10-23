<template>
    <div class="min-h-screen bg-gray-50" @dragover.prevent="noop">
        <!-- Global Drag Overlay -->
        <GlobalDragOverlay v-if="globalDragging" />

        <!-- Login View -->
        <LoginView
            v-if="!isAuthenticated"
            @login-success="handleLoginSuccess"
        />

        <!-- Drive View with Sidebar -->
        <DriveLayout v-else />
    </div>
</template>

<script setup>
import { onMounted, onUnmounted, computed } from "vue";
import { useAppStore } from "@/stores/appStore";
import LoginView from "@/components/auth/Login.vue";
import DriveLayout from "@/components/layout/DriveLayout.vue";
import GlobalDragOverlay from "@/components/ui/GlobalDragOverlay.vue";

const store = useAppStore();

// Usar computed para mejor reactividad
const isAuthenticated = computed(() => store.isAuthenticated);
const globalDragging = computed(() => store.globalDragging);

const noop = () => {};

const handleLoginSuccess = () => {
    console.log("Login successful - view should update");
    // Forzar actualización si es necesario
};

// Inicializar autenticación y drag & drop global
onMounted(() => {
    console.log("App mounted - initializing auth");
    store.initializeAuth();
    setupGlobalDragDrop();
});

onUnmounted(() => {
    cleanupGlobalDragDrop();
});

const setupGlobalDragDrop = () => {
    const enter = () => store.setGlobalDragging(true);
    const leave = () => store.setGlobalDragging(false);

    window.addEventListener("dragover", enter);
    window.addEventListener("dragleave", leave);

    window.addEventListener("drop", (e) => {
        e.preventDefault();
        store.setGlobalDragging(false);

        const file = e.dataTransfer.files[0];
        if (file) {
            store.setSelectedFile(file);
            store.setView("upload");
        }
    });
};

const cleanupGlobalDragDrop = () => {
    window.removeEventListener("dragover", () => {});
    window.removeEventListener("dragleave", () => {});
    window.removeEventListener("drop", () => {});
};
</script>
