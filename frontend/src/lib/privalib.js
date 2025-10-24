class DriveAPI {
  constructor() {
    this.baseUrl = "";
    this.token = "";
  }

  setConfig(baseUrl, token) {
    this.baseUrl = baseUrl;
    this.token = token;
  }

  async request(endpoint, options = {}) {
    const url = `${this.baseUrl}${endpoint}`;
    const config = {
      headers: {
        Authorization: `Bearer ${this.token}`,
        ...options.headers,
      },
      ...options,
    };

    const response = await fetch(url, config);
    const data = await response.json();

    if (!response.ok || !data.success) {
      throw new Error(data.message || `Error ${response.status}`);
    }

    return data;
  }

  async listFiles() {
    return this.request("/api/files/list");
  }

  async downloadFile(file) {
    return this.request(`/api/files/download/${file.id}`);
  }
  getDownloadUrl(file) {
    return `${this.baseUrl}/api/files/download/${file.id}`;
  }
  async uploadFile(file) {
    const mimeType = encodeURIComponent(
      file.type || "application/octet-stream",
    );

    return this.request(`/api/files/upload?mime=${mimeType}`, {
      method: "POST",
      headers: {
        "Content-Type": file.type || "application/octet-stream",
      },
      body: file,
    });
  }

  // Métodos utilitarios
  static getFileIcon(mime) {
    if (mime.startsWith("image/")) return "Image";
    if (mime.startsWith("video/")) return "Film";
    if (mime.startsWith("audio/")) return "Music";
    if (mime.includes("pdf") || mime.includes("document")) return "FileText";
    return "File";
  }

  static getFileColor(mime) {
    if (mime.startsWith("image/")) return "text-purple-500";
    if (mime.startsWith("video/")) return "text-red-500";
    if (mime.startsWith("audio/")) return "text-green-500";
    if (mime.includes("pdf")) return "text-red-600";
    return "text-gray-500";
  }

  static getMimeTypeLabel(mime) {
    const parts = mime.split("/");
    return parts[parts.length - 1].toUpperCase();
  }

  static getMimeTypeBadgeClass(mime) {
    if (mime.startsWith("image/")) return "bg-purple-100 text-purple-700";
    if (mime.startsWith("video/")) return "bg-red-100 text-red-700";
    if (mime.startsWith("audio/")) return "bg-green-100 text-green-700";
    if (mime.includes("pdf")) return "bg-orange-100 text-orange-700";
    return "bg-gray-100 text-gray-700";
  }

  static getFileName(file) {
    return `${file.id.substring(0, 8)}...`;
  }

  static formatFileSize(bytes) {
    if (bytes < 1024) return bytes + " B";
    if (bytes < 1024 * 1024) return (bytes / 1024).toFixed(2) + " KB";
    return (bytes / (1024 * 1024)).toFixed(2) + " MB";
  }
  static async login(username, password) {
    const response = await fetch(base_url);
    const data = await response.json();

    if (!response.ok || !data.success) {
      throw new Error(data.message || `Error ${response.status}`);
    }
    localStorage.setItem("token", data.token);
  }
}

export const driveAPI = new DriveAPI();
export { DriveAPI };
