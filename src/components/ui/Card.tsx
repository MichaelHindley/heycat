import { type HTMLAttributes, forwardRef } from "react";

export type CardVariant = "standard" | "interactive" | "status";

export interface CardProps extends HTMLAttributes<HTMLDivElement> {
  variant?: CardVariant;
  statusColor?: string;
}

const variantStyles: Record<CardVariant, string> = {
  standard: `
    bg-white
    rounded-[var(--radius-lg)]
    p-5
    shadow-sm
    border border-neutral-200
    transition-all duration-[var(--duration-normal)] ease-[var(--ease-default)]
    hover:shadow-md hover:border-neutral-300
  `,
  interactive: `
    bg-white
    rounded-[var(--radius-lg)]
    p-5
    shadow-sm
    border border-neutral-200
    cursor-pointer
    transition-all duration-[var(--duration-normal)] ease-[var(--ease-default)]
    hover:shadow-lg hover:border-heycat-orange
  `,
  status: `
    bg-white
    rounded-[var(--radius-lg)]
    p-5
    shadow-sm
    border border-neutral-200
    border-l-4
    transition-all duration-[var(--duration-normal)] ease-[var(--ease-default)]
  `,
};

export const Card = forwardRef<HTMLDivElement, CardProps>(
  ({ variant = "standard", statusColor, className = "", style, ...props }, ref) => {
    const statusStyle =
      variant === "status" && statusColor
        ? { borderLeftColor: statusColor, ...style }
        : style;

    return (
      <div
        ref={ref}
        className={`${variantStyles[variant]} ${className}`}
        style={statusStyle}
        {...props}
      />
    );
  }
);

Card.displayName = "Card";

// Card sub-components for composition
export interface CardHeaderProps extends HTMLAttributes<HTMLDivElement> {}

export const CardHeader = forwardRef<HTMLDivElement, CardHeaderProps>(
  ({ className = "", ...props }, ref) => (
    <div
      ref={ref}
      className={`mb-3 ${className}`}
      {...props}
    />
  )
);

CardHeader.displayName = "CardHeader";

export interface CardTitleProps extends HTMLAttributes<HTMLHeadingElement> {}

export const CardTitle = forwardRef<HTMLHeadingElement, CardTitleProps>(
  ({ className = "", ...props }, ref) => (
    <h3
      ref={ref}
      className={`text-lg font-semibold text-text-primary ${className}`}
      {...props}
    />
  )
);

CardTitle.displayName = "CardTitle";

export interface CardDescriptionProps extends HTMLAttributes<HTMLParagraphElement> {}

export const CardDescription = forwardRef<HTMLParagraphElement, CardDescriptionProps>(
  ({ className = "", ...props }, ref) => (
    <p
      ref={ref}
      className={`text-sm text-text-secondary ${className}`}
      {...props}
    />
  )
);

CardDescription.displayName = "CardDescription";

export interface CardContentProps extends HTMLAttributes<HTMLDivElement> {}

export const CardContent = forwardRef<HTMLDivElement, CardContentProps>(
  ({ className = "", ...props }, ref) => (
    <div
      ref={ref}
      className={className}
      {...props}
    />
  )
);

CardContent.displayName = "CardContent";

export interface CardFooterProps extends HTMLAttributes<HTMLDivElement> {}

export const CardFooter = forwardRef<HTMLDivElement, CardFooterProps>(
  ({ className = "", ...props }, ref) => (
    <div
      ref={ref}
      className={`mt-4 flex items-center gap-2 ${className}`}
      {...props}
    />
  )
);

CardFooter.displayName = "CardFooter";
