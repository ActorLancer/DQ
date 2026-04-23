"use client";

import { create } from "zustand";

interface PortalShellState {
  authDialogOpen: boolean;
  setAuthDialogOpen: (open: boolean) => void;
}

export const usePortalShellStore = create<PortalShellState>((set) => ({
  authDialogOpen: false,
  setAuthDialogOpen: (authDialogOpen) => set({ authDialogOpen }),
}));
