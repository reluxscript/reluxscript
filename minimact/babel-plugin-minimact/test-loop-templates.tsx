import { useState } from '@minimact/core';

interface Todo {
  id: number;
  text: string;
  done: boolean;
}

/**
 * Test component for loop template extraction
 *
 * This should generate loop templates for:
 * 1. todos.map() - simple list with conditional rendering
 * 2. FAQs pattern - accordion items
 */
export function TodoList() {
  const [todos, setTodos] = useState<Todo[]>([
    { id: 1, text: 'Buy milk', done: false },
    { id: 2, text: 'Walk dog', done: true }
  ]);
  const [input, setInput] = useState('');

  return (
    <div className="todo-list">
      <h1>Todo List</h1>

      <ul>
        {todos.map(todo => (
          <li key={todo.id} className={todo.done ? 'done' : 'pending'}>
            <span>{todo.text}</span>
            <span>{todo.done ? '✓' : '○'}</span>
          </li>
        ))}
      </ul>
    </div>
  );
}

/**
 * FAQ Accordion Pattern
 * Should generate loop template for FAQs.map()
 */
export function FAQPage() {
  const [faqs, setFaqs] = useState([
    { id: 1, question: 'What is Minimact?', answer: 'A server-side React framework' },
    { id: 2, question: 'How does it work?', answer: 'It renders React on the server' }
  ]);
  const [openIndex, setOpenIndex] = useState<number | null>(null);

  return (
    <div className="faq-page">
      <h1>FAQs</h1>

      {faqs.map((item, index) => (
        <div key={item.id} className="faq-item">
          <button
            onClick={() => setOpenIndex(openIndex === index ? null : index)}
            className={openIndex === index ? 'open' : 'closed'}
          >
            {item.question}
          </button>
          {openIndex === index && (
            <div className="answer">{item.answer}</div>
          )}
        </div>
      ))}
    </div>
  );
}
