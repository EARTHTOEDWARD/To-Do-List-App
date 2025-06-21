import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/tauri";
import "./App.css";

interface Task {
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

function App() {
  const [tasks, setTasks] = useState<Task[]>([]);
  const [newTaskTitle, setNewTaskTitle] = useState("");
  const [newTaskDescription, setNewTaskDescription] = useState("");
  const [loading, setLoading] = useState(false);

  useEffect(() => {
    initDatabase();
    loadTasks();
  }, []);

  const initDatabase = async () => {
    try {
      const result = await invoke<string>("init_database");
      console.log(result);
    } catch (error) {
      console.error("Failed to initialize database:", error);
    }
  };

  const loadTasks = async () => {
    try {
      const taskData = await invoke<Task[]>("get_tasks");
      setTasks(taskData);
    } catch (error) {
      console.error("Failed to load tasks:", error);
    }
  };

  const addTask = async () => {
    if (!newTaskTitle.trim()) return;
    
    setLoading(true);
    try {
      await invoke("add_task", {
        title: newTaskTitle,
        description: newTaskDescription || null,
        priority: "medium",
      });
      
      setNewTaskTitle("");
      setNewTaskDescription("");
      await loadTasks();
    } catch (error) {
      console.error("Failed to add task:", error);
    } finally {
      setLoading(false);
    }
  };

  const completeTask = async (taskId: string) => {
    try {
      await invoke("complete_task", { taskId });
      await loadTasks();
    } catch (error) {
      console.error("Failed to complete task:", error);
    }
  };

  const getPriorityColor = (priority: string) => {
    switch (priority) {
      case "high": return "🔴";
      case "medium": return "🟡";
      case "low": return "🟢";
      default: return "⚪";
    }
  };

  return (
    <div className="container">
      <h1>📊 Graph-OS</h1>
      <p>Local-first visual To-Do & knowledge graph</p>

      <div className="add-task">
        <h2>Add New Task</h2>
        <input
          type="text"
          placeholder="Task title..."
          value={newTaskTitle}
          onChange={(e) => setNewTaskTitle(e.target.value)}
          onKeyPress={(e) => e.key === "Enter" && addTask()}
        />
        <textarea
          placeholder="Description (optional)..."
          value={newTaskDescription}
          onChange={(e) => setNewTaskDescription(e.target.value)}
          rows={3}
        />
        <button onClick={addTask} disabled={loading || !newTaskTitle.trim()}>
          {loading ? "Adding..." : "Add Task"}
        </button>
      </div>

      <div className="tasks">
        <h2>📋 Tasks ({tasks.length})</h2>
        {tasks.length === 0 ? (
          <p className="no-tasks">No tasks yet. Add one above!</p>
        ) : (
          <div className="task-list">
            {tasks.map((task) => (
              <div
                key={task.id}
                className={`task ${task.payload.completed ? "completed" : ""}`}
              >
                <div className="task-header">
                  <span className="task-status">
                    {task.payload.completed ? "✅" : "⏳"}
                  </span>
                  <span className="task-priority">
                    {getPriorityColor(task.payload.priority)}
                  </span>
                  <h3 className="task-title">{task.payload.title}</h3>
                  {!task.payload.completed && (
                    <button
                      className="complete-btn"
                      onClick={() => completeTask(task.id)}
                    >
                      Complete
                    </button>
                  )}
                </div>
                
                {task.payload.description && (
                  <p className="task-description">{task.payload.description}</p>
                )}
                
                <div className="task-meta">
                  <span className="task-id">ID: {task.id.slice(0, 8)}...</span>
                  <span className="task-created">
                    Created: {new Date(task.created).toLocaleDateString()}
                  </span>
                </div>
              </div>
            ))}
          </div>
        )}
      </div>
    </div>
  );
}

export default App;
