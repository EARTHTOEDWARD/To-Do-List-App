import { invoke } from "@tauri-apps/api/tauri";

export interface Task {
  id: string;
  payload: {
    title: string;
    description?: string;
    completed: boolean;
    priority: string;
  };
  created: string;
  updated?: string;
}

export const api = {
  async initDatabase(): Promise<string> {
    return await invoke<string>("init_database");
  },

  async addTask(title: string, description?: string): Promise<string> {
    return await invoke<string>("add_task", {
      title,
      description: description || null,
    });
  },

  async getTasks(): Promise<Task[]> {
    return await invoke<Task[]>("get_tasks");
  },

  async completeTask(taskId: string): Promise<string> {
    return await invoke<string>("complete_task", { taskId });
  },
};
