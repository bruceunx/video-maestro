import type * as React from "react";

import ReactMarkdown from "react-markdown";
import remarkGfm from "remark-gfm";

interface StreamTextProps {
  content: string;
}

const StreamText: React.FC<StreamTextProps> = ({ content }) => {
  return (
    <div className="prose p-5">
      <ReactMarkdown
        remarkPlugins={[remarkGfm]}
        components={{
          h1: ({ ...props }) => (
            <h1 className="text-3xl font-bold text-blue-600" {...props} />
          ),
          h2: ({ ...props }) => (
            <h2 className="text-2xl font-semibold text-blue-500" {...props} />
          ),
          p: ({ ...props }) => (
            <p className="text-gray-700 leading-relaxed" {...props} />
          ),
          a: ({ ...props }) => (
            <a
              className="text-blue-400 underline hover:text-blue-600"
              {...props}
            />
          ),
          ul: ({ ...props }) => (
            <ul className="list-disc pl-5 space-y-2" {...props} />
          ),
          table: ({ ...props }) => (
            <table
              className="table-auto border-collapse border border-gray-300"
              {...props}
            />
          ),
          th: ({ ...props }) => (
            <th
              className="border border-gray-300 bg-gray-200 px-4 py-2 text-left"
              {...props}
            />
          ),
          td: ({ ...props }) => (
            <td className="border border-gray-300 px-4 py-2" {...props} />
          ),
          ol: ({ ...props }) => (
            <ol className="list-decimal pl-5 space-y-2" {...props} />
          ),
          li: ({ ...props }) => (
            <li className="text-gray-700 leading-relaxed" {...props} />
          ),
          blockquote: ({ ...props }) => (
            <blockquote
              className="border-l-4 border-blue-400 pl-4 italic text-gray-600"
              {...props}
            />
          ),
        }}
      >
        {content}
      </ReactMarkdown>
    </div>
  );
};

export default StreamText;
