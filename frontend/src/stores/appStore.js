import { defineStore } from "pinia";
import { ref, computed } from "vue";
import { driveAPI } from "@/lib/privalib";

export const useAppStore = defineStore("app", () => {
  // Estado de autenticación
  const isAuthenticated = ref(false);
  const serverUrl = ref("http://localhost:5830");
  const token = ref("");
  const loading = ref(false);
  const error = ref("");

  // Estado de la UI
  const sidebarOpen = ref(true);
  const currentView = ref("files");
  const isDragging = ref(false);
  const globalDragging = ref(false);

  // Estado de archivos
  const files = ref([]);
  const selectedFile = ref(null);
  const uploading = ref(false);
  const uploadMessage = ref("");
  const uploadSuccess = ref(false);

  // Getters computados
  const fileStats = computed(() => ({
    total: files.value.length,
    images: files.value.filter((file) => file.mime.startsWith("image")).length,
    videos: files.value.filter((file) => file.mime.startsWith("video")).length,
    audio: files.value.filter((file) => file.mime.startsWith("audio")).length,
    documents: files.value.filter(
      (file) => file.mime.includes("pdf") || file.mime.includes("document"),
    ).length,
  }));

  const initializeAuth = async () => {
    const storedToken = localStorage.getItem("token");
    const storedUrl = localStorage.getItem("serverUrl"); // Añade esto

    if (storedToken && storedUrl) {
      token.value = storedToken;
      serverUrl.value = storedUrl;
      try {
        // CONFIGURA la API antes de hacer la petición
        driveAPI.setConfig(storedUrl, storedToken);
        await fetchFiles();
        isAuthenticated.value = true;
      } catch (err) {
        console.error("Error initializing auth:", err);
        localStorage.removeItem("token");
        localStorage.removeItem("serverUrl"); // Limpia también la URL
        isAuthenticated.value = false;
      }
    }
  };

  const login = async (url, authToken) => {
    loading.value = true;
    error.value = "";
    serverUrl.value = url;
    token.value = authToken;

    try {
      console.log("Setting API config with:", url, authToken);

      // Asegúrate de que la URL esté bien formada
      const formattedUrl = url.endsWith("/") ? url.slice(0, -1) : url;
      driveAPI.setConfig(formattedUrl, authToken);

      console.log("Fetching files...");
      await fetchFiles();

      isAuthenticated.value = true;
      localStorage.setItem("token", authToken);
      localStorage.setItem("serverUrl", formattedUrl);

      console.log("Login successful, isAuthenticated:", isAuthenticated.value);
    } catch (err) {
      console.error("Login error in store:", err);
      error.value = err.message;
      isAuthenticated.value = false;
      throw err;
    } finally {
      loading.value = false;
    }
  };

  const logout = () => {
    isAuthenticated.value = false;
    files.value = [];
    token.value = "";
    error.value = "";
    currentView.value = "files";
    localStorage.removeItem("token");
  };

  const fetchFiles = async () => {
    loading.value = true;
    error.value = "";

    try {
      const data = await driveAPI.listFiles();
      files.value = data.files || [];
    } catch (err) {
      error.value = err.message;
      if (!isAuthenticated.value) throw err;
    } finally {
      loading.value = false;
    }
  };

  const uploadFile = async (file) => {
    uploading.value = true;
    uploadMessage.value = "";

    try {
      const data = await driveAPI.uploadFile(file);
      uploadMessage.value = data.message || "Archivo subido exitosamente";
      uploadSuccess.value = true;
      await fetchFiles();
      return data;
    } catch (err) {
      uploadMessage.value = err.message;
      uploadSuccess.value = false;
      throw err;
    } finally {
      uploading.value = false;
    }
  };

  const setSelectedFile = (file) => {
    selectedFile.value = file;
    uploadMessage.value = "";
  };

  const clearSelectedFile = () => {
    selectedFile.value = null;
    uploadMessage.value = "";
  };

  const setView = (view) => {
    currentView.value = view;
  };

  const toggleSidebar = () => {
    sidebarOpen.value = !sidebarOpen.value;
  };

  const setDragging = (value) => {
    isDragging.value = value;
  };

  const setGlobalDragging = (value) => {
    globalDragging.value = value;
  };

  return {
    // Estado
    isAuthenticated,
    serverUrl,
    token,
    loading,
    error,
    sidebarOpen,
    currentView,
    isDragging,
    globalDragging,
    files,
    selectedFile,
    uploading,
    uploadMessage,
    uploadSuccess,

    // Getters
    fileStats,

    // Acciones
    initializeAuth,
    login,
    logout,
    fetchFiles,
    uploadFile,
    setSelectedFile,
    clearSelectedFile,
    setView,
    toggleSidebar,
    setDragging,
    setGlobalDragging,
  };
});
