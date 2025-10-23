<template>
    <div class="flex h-screen overflow-hidden">
        <Sidebar />

        <!-- Main Content -->
        <div class="flex-1 flex flex-col overflow-hidden">
            <Header />

            <main class="flex-1 overflow-y-auto p-6">
                <FilesView v-if="currentView === 'files'" />
                <UploadView v-else-if="currentView === 'upload'" />
                <StatsView v-else-if="currentView === 'stats'" />
            </main>
        </div>
    </div>
</template>

<script setup>
import { useAppStore } from "@/stores/appStore";
import Sidebar from "./Sidebar.vue";
import Header from "./Header.vue";
import FilesView from "@/components/views/FilesView.vue";
import UploadView from "@/components/views/UploadView.vue";
import StatsView from "@/components/views/StatsView.vue";

const store = useAppStore();
const { currentView } = store;
</script>
