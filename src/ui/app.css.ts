// Will need:
// https://github.com/Qix-/color#readme

import {
  composeStyles,
  createTheme,
  globalStyle,
  keyframes,
  style,
} from "@vanilla-extract/css";

export const [darkThemeClass, vars] = createTheme({
  textColor: "#c5c6c7",
  backgroundColor: "#232323",

  textColorReverse: "#D0D0D0",
  backgroundColorHighlight: "#4F5C6C",

  backgroundColorTransparent: "#1F1F1FEF",
  backgroundColorSelected: "#4A4A4A",
  backgroundColorButton: "#4A4A4A",
  backgroundColorHover: "#333333",

  backgroundColorBroadcast:
    "linear-gradient(45deg, #3E3E68, #3E3E68 30%, #5E5B30 70%, #5E5B30)",

  backgroundColorBroadcastActive:
    "linear-gradient(45deg, #0057B8, #0057B8 30%, #FFD700 70%, #FFD700)",

  borderColor: "#3e3e3e",
  shadowColor: "rgba(102, 102, 102, 0.4)",
});

export const lightThemeClass = createTheme(vars, {
  textColor: "#1d1f21",
  backgroundColor: "#f5f5f5",

  textColorReverse: "#1a1c1e",
  backgroundColorHighlight: "#C0D4EE",

  backgroundColorTransparent: "#FFFFFFEF",
  backgroundColorSelected: "#D1D1D1",
  backgroundColorButton: "#D1D1D1",
  backgroundColorHover: "#E2E2E2",

  backgroundColorBroadcast:
    "linear-gradient(45deg, #B0CFF7, #B0CFF7 30%, #FBF9D6 70%, #FBF9D6)",

  backgroundColorBroadcastActive:
    "linear-gradient(45deg, #00A5FF, #00A5FF 30%, #FFF375 70%, #FFF375)",

  borderColor: "#ccc",
  shadowColor: "rgba(0, 0, 0, 0.2)",
});

globalStyle("html, body", {
  fontFamily: "ui-monospace",
  backgroundColor: "transparent",
  padding: "10px",
  letterSpacing: "-0.02ch",
  height: "100%",
  overflow: "hidden",
});

globalStyle("main", {
  color: vars.textColor,
  backgroundColor: vars.backgroundColorTransparent,
  borderRadius: "10px",
  boxShadow: "0 0 10px rgba(0, 0, 0, 0.5)",
  width: "100%",
  height: "100%",
  overflow: "auto",
  display: "flex",
  flexDirection: "column",
});

globalStyle(".hoverable", {
  borderRadius: "4px",
  padding: "4px",
});

globalStyle(".hoverable:hover", {
  backgroundColor: vars.backgroundColorHover,
});

globalStyle("input", {
  border: "none",
  width: "100%",
  outline: "none",
});

globalStyle(".markdown", {
  padding: "0 2ch",
  verticalAlign: "middle",
});

globalStyle("code", {
  backgroundColor: vars.backgroundColorHover,
  borderRadius: "2px",
  padding: "0 0.5ch",
});

globalStyle(".markdown input", {
  width: "auto", // Revert back to default width
});

globalStyle(".markdown blockquote", {
  paddingLeft: "2ch",
  borderLeft: ("4px solid " + vars.borderColor),
  marginBlockStart: "0",
  marginBlockEnd: "0",
  marginInlineStart: "0",
  marginInlineEnd: "0",
});

globalStyle("a", {
  color: vars.textColor,
  textDecoration: "underline",
  textDecorationColor: vars.borderColor,
  textUnderlineOffset: "3px",
});

globalStyle(".terserow:hover", {
  backgroundColor: vars.backgroundColorHover,
});

globalStyle(".terserow.hover", {
  backgroundColor: vars.backgroundColorHover,
});

globalStyle(".terserow.selected", {
  backgroundColor: vars.backgroundColorSelected,
});

globalStyle(".terserow.highlight", {
  backgroundColor: vars.backgroundColorHighlight,
  color: vars.textColorReverse,
});

export const previewItem = style({
  padding: "0.25lh 0",
  selectors: {
    "&.not-active": {
      opacity: "0.3",
      filter: "grayscale(30%)",
    },
    "&.not-active:hover": {
      opacity: "0.6",
      filter: "grayscale(60%)",
      backgroundColor: vars.backgroundColor,
    },
    "&.active": {
      boxShadow: `0 0 6px ${vars.shadowColor}`,
      backgroundColor: vars.backgroundColor,
    },
  },
});

export const footer = style({
  display: "flex",
  alignItems: "center",
  height: "5ch",
  boxShadow: "0 0 4px " + vars.shadowColor,
  fontSize: "0.9rem",
  backgroundColor: vars.backgroundColor,
  padding: "1ch",
  paddingLeft: "2ch",
  paddingRight: "1ch",
  justifyContent: "space-between",
});

export const overlay = style({
  boxShadow: "0 0 6px " + vars.shadowColor,
  backgroundColor: vars.backgroundColorTransparent,
});

export const card = style({
  backgroundColor: vars.backgroundColorTransparent,
  height: "100%",
  width: "auto",
  display: "flex",
  borderRadius: "1ch",
  flexDirection: "column",
});

export const borderRight = style({
  borderRightWidth: "1px",
  borderRightStyle: "solid",
  borderRightColor: vars.borderColor,
});

export const border = style({
  borderWidth: "1px",
  borderStyle: "solid",
  borderColor: vars.borderColor,
});

export const borderBottom = style({
  borderBottomWidth: "1px",
  borderBottomStyle: "solid",
  borderBottomColor: vars.borderColor,
});

export const iconStyle = style({
  display: "inline-block",
  textAlign: "center",
  backgroundColor: vars.backgroundColorButton,
  borderRadius: "5px",
  paddingLeft: "1ch",
  paddingRight: "1ch",
  paddingBottom: "0.2ch",
});

const swirlAnimation = keyframes({
  "0%": {
    backgroundPosition: "0% 0%",
  },
  "100%": {
    backgroundPosition: "10% 10%",
  },
});

export const button = style({
  borderRadius: "4px",
  padding: "4px",
});

export const enchantedForestGradient = composeStyles(
  button,
  style({
    transition: "0.2s",
    ":hover": {
      borderColor: vars.backgroundColorTransparent,
      backgroundImage: vars.backgroundColorBroadcast,
      boxShadow: "0 0 2px " + vars.textColor,
      backgroundSize: "200% auto",
    },
  }),
);

export const enchantedForestGradientActive = composeStyles(
  button,
  style({
    backgroundImage: vars.backgroundColorBroadcastActive,
    borderColor: vars.backgroundColorTransparent,
    backgroundSize: "200% auto",
    transition: "0.2s",
    boxShadow: "0 0 1px " + vars.textColor,
    animation: `${swirlAnimation} 1s infinite alternate`,
    ":hover": {
      boxShadow: "0 0 5px " + vars.textColor,
    },
  }),
);
