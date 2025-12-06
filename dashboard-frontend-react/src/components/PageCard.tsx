import React from "react";

interface Props {
  title: string;
  children?: React.ReactNode;
}

const PageCard: React.FC<Props> = ({ title, children }) => {
  return (
    <div style={{ padding: "24px" }}>
      <div
        style={{
          width: "100%",
          background: "#ffffff",
          borderRadius: 12,
          padding: "28px 32px",
          boxShadow: "0 1px 2px rgba(0,0,0,0.05)",
        }}
      >
        <h2 style={{ margin: 0, fontSize: 24, fontWeight: 600 }}>{title}</h2>

        <div style={{ marginTop: 24 }}>{children}</div>
      </div>
    </div>
  );
};

export default PageCard;
