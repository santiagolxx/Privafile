<template>
    <div class="video-player-container bg-black rounded-lg overflow-hidden">
        <video
            ref="videoRef"
            :poster="poster"
            controls
            class="w-full h-full"
            :title="title"
        >
            <source :src="blobUrl" :type="sourceType" />
            Tu navegador no soporta el elemento de video.
        </video>
    </div>
</template>

<script setup>
import { onMounted, ref, computed, watch, onUnmounted } from "vue";

const props = defineProps({
    videoUrl: {
        type: String,
        required: true,
    },
    title: {
        type: String,
        default: "Video",
    },
    poster: {
        type: String,
        default: "",
    },
    token: {
        type: String,
        required: true,
    },
});

const videoRef = ref(null);
const blobUrl = ref("");

const sourceType = computed(() => {
    const url = props.videoUrl.toLowerCase();
    if (url.includes(".mp4")) return "video/mp4";
    if (url.includes(".webm")) return "video/webm";
    if (url.includes(".ogg")) return "video/ogg";
    return "video/mp4";
});

const loadVideoWithAuth = async () => {
    // Limpiar URL anterior si existe
    if (blobUrl.value) {
        URL.revokeObjectURL(blobUrl.value);
    }

    try {
        const response = await fetch(props.videoUrl, {
            headers: {
                Authorization: `Bearer ${props.token}`,
            },
        });

        if (!response.ok) {
            throw new Error(`HTTP error! status: ${response.status}`);
        }

        const blob = await response.blob();
        blobUrl.value = URL.createObjectURL(blob);
    } catch (error) {
        console.error("Error loading video:", error);
    }
};

onMounted(() => {
    loadVideoWithAuth();
});

// Recargar cuando cambie la URL o el token
watch([() => props.videoUrl, () => props.token], () => {
    loadVideoWithAuth();
});

// Limpiar al desmontar el componente
onUnmounted(() => {
    if (blobUrl.value) {
        URL.revokeObjectURL(blobUrl.value);
    }
});
</script>

<style scoped>
.video-player-container {
    width: 100%;
    max-width: 800px;
    aspect-ratio: 16 / 9;
}
</style>
