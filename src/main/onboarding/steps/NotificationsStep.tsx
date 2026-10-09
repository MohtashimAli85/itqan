import { useMutation } from "@tanstack/react-query";
import {
  isPermissionGranted,
  requestPermission,
} from "@tauri-apps/plugin-notification";
import { Button } from "@/shared/components/ui/button";

async function askForPermission() {
  if (await isPermissionGranted()) return "granted";
  return requestPermission();
}

export function NotificationsStep() {
  const permission = useMutation({ mutationFn: askForPermission });
  const granted = permission.data === "granted";

  return (
    <div className="flex flex-col gap-6">
      <div className="flex flex-col gap-1">
        <h2 className="font-heading text-2xl font-bold">
          A backup for reminders
        </h2>
        <p className="text-text-secondary text-sm">
          Reminders come through me. When I'm hidden, I use system notifications
          so you never miss your medicine or a promise.
        </p>
      </div>
      <div className="flex items-center gap-3">
        <Button
          type="button"
          variant="secondary"
          disabled={permission.isPending || granted}
          onClick={() => permission.mutate()}
        >
          {granted ? "Notifications allowed" : "Allow notifications"}
        </Button>
        {permission.data === "denied" && (
          <p className="text-caption text-sm">
            No problem. You can allow them later in System Settings.
          </p>
        )}
      </div>
    </div>
  );
}
