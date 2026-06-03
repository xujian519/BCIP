import type { FC } from 'react';
import { motion } from 'framer-motion';
import { Github, FileText, AlertCircle, ExternalLink } from 'lucide-react';

const containerVariants = {
  hidden: { opacity: 0 },
  show: {
    opacity: 1,
    transition: { staggerChildren: 0.1 },
  },
};

const itemVariants = {
  hidden: { opacity: 0, y: 12 },
  show: { opacity: 1, y: 0, transition: { duration: 0.3, ease: 'easeOut' as const } },
};

const AboutSettings: FC = () => {
  const links = [
    { label: '官网', href: '#', icon: <ExternalLink size={12} /> },
    { label: '文档', href: '#', icon: <FileText size={12} /> },
    { label: 'GitHub', href: '#', icon: <Github size={12} /> },
    { label: '反馈问题', href: '#', icon: <AlertCircle size={12} /> },
  ];

  return (
    <motion.div
      variants={containerVariants}
      initial="hidden"
      animate="show"
      className="flex flex-col items-center"
      style={{ padding: '32px 24px' }}
    >
      {/* App Icon */}
      <motion.div
        variants={itemVariants}
        initial={{ scale: 0.9, opacity: 0 }}
        animate={{ scale: 1, opacity: 1 }}
        transition={{ duration: 0.3, ease: [0.34, 1.56, 0.64, 1] as [number, number, number, number] }}
      >
        <img
          src="/app-icon.png"
          alt="云熙智能体"
          style={{
            width: 80,
            height: 80,
            borderRadius: 16,
            boxShadow: '0 4px 16px rgba(0,0,0,0.1)',
            objectFit: 'cover',
          }}
        />
      </motion.div>

      {/* App Name */}
      <motion.h2
        variants={itemVariants}
        style={{
          fontSize: 22,
          fontWeight: 600,
          color: 'var(--text-primary)',
          letterSpacing: '-0.015em',
          lineHeight: 1.3,
          marginTop: 16,
        }}
      >
        云熙智能体
      </motion.h2>

      {/* Version */}
      <motion.span
        variants={itemVariants}
        style={{
          fontSize: 12,
          color: 'var(--text-secondary)',
          marginTop: 4,
          fontFamily: 'JetBrains Mono, monospace',
        }}
      >
        v2.1.0 (Build 20241215)
      </motion.span>

      {/* Description */}
      <motion.p
        variants={itemVariants}
        style={{
          fontSize: 12,
          color: 'var(--text-secondary)',
          lineHeight: 1.6,
          textAlign: 'center',
          maxWidth: 360,
          marginTop: 12,
        }}
      >
        基于 DeepSeek 大语言模型的专业专利智能助手，提供专利检索、分析、撰写等全流程 AI 辅助。
      </motion.p>

      {/* Links */}
      <motion.div
        variants={itemVariants}
        className="flex items-center gap-1"
        style={{ marginTop: 20 }}
      >
        {links.map((link, idx) => (
          <span key={link.label} className="flex items-center">
            <a
              href={link.href}
              className="flex items-center gap-1 px-2 py-1 transition-colors"
              style={{
                fontSize: 12,
                color: 'var(--accent-primary)',
                textDecoration: 'none',
                borderRadius: 4,
              }}
              onMouseEnter={(e) => {
                e.currentTarget.style.textDecoration = 'underline';
                e.currentTarget.style.backgroundColor = 'var(--accent-primary-muted)';
              }}
              onMouseLeave={(e) => {
                e.currentTarget.style.textDecoration = 'none';
                e.currentTarget.style.backgroundColor = 'transparent';
              }}
            >
              {link.label}
            </a>
            {idx < links.length - 1 && (
              <span
                style={{
                  color: 'var(--text-tertiary)',
                  fontSize: 12,
                  margin: '0 2px',
                }}
              >
                ·
              </span>
            )}
          </span>
        ))}
      </motion.div>

      {/* Tech Stack */}
      <motion.span
        variants={itemVariants}
        style={{
          fontSize: 11,
          color: 'var(--text-tertiary)',
          letterSpacing: '0.01em',
          marginTop: 16,
        }}
      >
        Powered by DeepSeek · React · Tauri
      </motion.span>

      {/* Copyright */}
      <motion.span
        variants={itemVariants}
        style={{
          fontSize: 11,
          color: 'var(--text-tertiary)',
          letterSpacing: '0.01em',
          marginTop: 8,
        }}
      >
        &copy; 2024 YunXi Agent. All rights reserved.
      </motion.span>

      {/* MIT License */}
      <motion.div
        variants={itemVariants}
        className="mt-4 px-4 py-3"
        style={{
          borderRadius: 8,
          backgroundColor: 'var(--bg-surface)',
          border: '1px solid var(--border-secondary)',
          maxWidth: 400,
          width: '100%',
        }}
      >
        <p
          style={{
            fontSize: 10,
            color: 'var(--text-tertiary)',
            lineHeight: 1.5,
            textAlign: 'center',
            fontFamily: 'JetBrains Mono, monospace',
          }}
        >
          MIT License
        </p>
        <p
          style={{
            fontSize: 10,
            color: 'var(--text-tertiary)',
            lineHeight: 1.5,
            textAlign: 'center',
            marginTop: 4,
          }}
        >
          Permission is hereby granted, free of charge, to any person obtaining a copy of this
          software and associated documentation files.
        </p>
      </motion.div>
    </motion.div>
  );
};

export default AboutSettings;
