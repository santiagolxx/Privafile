<template>
    <div class="flex items-center justify-center min-h-screen p-4">
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

                <form @submit.prevent="handleSubmit" class="space-y-4">
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

                <p v-if="error" class="mt-4 text-sm text-red-600 text-center">
                    {{ error }}
                </p>
            </div>
        </div>
    </div>
</template>

<script setup>
import { ref } from "vue";
import { HardDrive } from "lucide-vue-next";
import { useAppStore } from "@/stores/appStore";

const store = useAppStore();
const { login } = store;

const serverUrl = ref("http://localhost:5830");
const token = ref("");
const loading = ref(false);
const error = ref("");

const emit = defineEmits(["login-success"]);

const handleSubmit = async () => {
    loading.value = true;
    error.value = "";

    try {
        await login(serverUrl.value, token.value);
        console.log("Login successful in component");
        emit("login-success"); // Emitir evento
    } catch (err) {
        console.error("Login error in component:", err);
        error.value = err.message || "Error desconocido";
    } finally {
        loading.value = false;
    }
};
</script>
