import * as core from '@actions/core';

interface Annotation {
  start_line: number;
  end_line: number;
  start_column: number;
  end_column: number;
  level: string;
  title: string;
  message: string;
  path: string;
}

export function parseAnnotation(line: string): Annotation {
  const parts = line.split('||');
  return {
    start_line: parseInt(parts[0]),
    end_line: parseInt(parts[1]),
    start_column: parseInt(parts[2]),
    end_column: parseInt(parts[3]),
    level: parts[4],
    title: parts[5],
    message: parts[6],
    path: parts[7]
  };
}

export function annotationParams(
  annotation: Annotation
): core.AnnotationProperties {
  const props: core.AnnotationProperties = {
    file: annotation.path,
    title: annotation.title,
    startLine: annotation.start_line,
    endLine: annotation.end_line
  };
  if (annotation.start_line === annotation.end_line) {
    props.startColumn = annotation.start_column;
    props.endColumn = annotation.end_column;
  }
  return props;
}
