<template>
    <div
        v-if="show"
        class="fixed inset-0 bg-black bg-opacity-75 flex items-center justify-center z-50"
        @click.self="close"
    >
        <div class="bg-white rounded-lg overflow-hidden max-w-4xl w-full mx-4">
            <div
                class="p-4 border-b border-gray-200 flex justify-between items-center"
            >
                <h3 class="text-lg font-semibold">{{ title }}</h3>
                <button
                    @click="close"
                    class="text-gray-500 hover:text-gray-700"
                >
                    <X class="w-6 h-6" />
                </button>
            </div>
            <div class="p-4">
                <VideoPlayer :video-url="videoUrl" :title="title" />
            </div>
        </div>
    </div>
</template>

<script setup>
import { X } from "lucide-vue-next";
import VideoPlayer from "./VideoPlayer.vue";

defineProps({
    show: Boolean,
    videoUrl: String,
    title: String,
});

const emit = defineEmits(["close"]);

const close = () => {
    emit("close");
};
</script>
