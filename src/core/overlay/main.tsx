import React from "react";
import ReactDOM from "react-dom/client";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import "@/shared/styles/globals.css";
import { Listeners } from "@/core/registry/Listeners";
import { Overlay } from "./Overlay";

const queryClient = new QueryClient();

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <QueryClientProvider client={queryClient}>
      <Listeners surface="overlay" />
      <Overlay />
    </QueryClientProvider>
  </React.StrictMode>,
);
