/**
 * 共享动画常量
 *
 * 统一管理 framer-motion 使用的 cubic-bezier 曲线，
 * 避免跨文件重复定义。
 */

/** 面板弹出/收起曲线（material design 标准 acceleration） */
export const easePanel: [number, number, number, number] = [0.4, 0, 0.2, 1];

/** 页面过渡曲线（柔和减速） */
export const easePageOut: [number, number, number, number] = [0.16, 1, 0.3, 1];
