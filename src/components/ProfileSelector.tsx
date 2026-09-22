import { memo } from "react";
import type { ProfileId, ProfileDef } from "../lib/types";

interface Props {
  profiles: Record<ProfileId, ProfileDef>;
  active: ProfileId;
  onChange: (id: ProfileId) => void;
}

const ORDER: ProfileId[] = ["idle", "low", "medium", "high", "extreme"];

export const ProfileSelector = memo(function ProfileSelector({ profiles, active, onChange }: Props) {
  return (
    <div className="profile-row">
      {ORDER.map((id) => {
        const p = profiles[id];
        return (
          <button
            key={id}
            className={`profile-chip ${active === id ? "active" : ""}`}
            onClick={() => onChange(id)}
            type="button"
          >
            <strong>{p.label}</strong>
            <span>
              CPU {Math.round(p.cpu_intensity * 100)}% · GPU {Math.round(p.gpu_intensity * 100)}%
            </span>
          </button>
        );
      })}
    </div>
  );
});
