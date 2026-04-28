"use client";

import { create } from "zustand";

interface ConsoleShellState {
  authDialogOpen: boolean;
  setAuthDialogOpen: (open: boolean) => void;
}

export const useConsoleShellStore = create<ConsoleShellState>((set) => ({
  authDialogOpen: false,
  setAuthDialogOpen: (authDialogOpen) => set({ authDialogOpen }),
}));
