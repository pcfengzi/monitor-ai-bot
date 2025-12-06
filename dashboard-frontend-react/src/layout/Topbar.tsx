// src/layout/Topbar.tsx
import React from "react";
import "../App.css";

interface TopbarProps {
  onToggleSidebar?: () => void;
}

const Topbar: React.FC<TopbarProps> = ({ onToggleSidebar }) => {
  return (
    <header className="topbar">
      <div className="topbar-left">
        <button className="topbar-menu-btn" onClick={onToggleSidebar}>
          ☰
        </button>
        <div className="topbar-logo">Monitor AI Bot</div>
        <span className="topbar-env-tag">DEV</span>
      </div>

      <div className="topbar-right">
        <input
          className="topbar-search"
          placeholder="搜索 指标 / 告警 / 工作流..."
        />
        <button className="topbar-icon-btn" title="设置">
          ⚙
        </button>
        <div className="topbar-user">
          <span className="topbar-avatar">F</span>
          <span className="topbar-username">feng yuan</span>
        </div>
      </div>
    </header>
  );
};

export default Topbar;
