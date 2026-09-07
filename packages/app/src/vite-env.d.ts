/// <reference types="vite/client" />

// Vite's `?worker` import returns a constructor for a Worker built from that
// module. The worker file ships inside the application; nothing is fetched.
declare module '*?worker' {
  const WorkerConstructor: new () => Worker;
  export default WorkerConstructor;
}
