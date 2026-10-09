import { create } from "zustand";

type MainUiState = {
  selectedTaskId: number | null;
  selectTask: (id: number | null) => void;
};

export const useMainUi = create<MainUiState>((set) => ({
  selectedTaskId: null,
  selectTask: (id) => set({ selectedTaskId: id }),
}));
