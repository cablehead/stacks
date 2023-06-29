interface ImportMeta {
  hot?: {
    accept: (callback?: () => void) => void;
    dispose: (callback: () => void) => void;
  };
}
