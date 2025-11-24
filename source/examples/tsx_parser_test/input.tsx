// Test TSX file for ReluxScript parser test

interface ViewModel {
  name: string;
  age: number;
  isActive: boolean;
  tags: string[];
}

interface UserProfile {
  id: string;
  email: string;
  preferences?: {
    theme: string;
    language: string;
  };
}

function Component() {
  const state = useState<string>("hello");
  const count = useState<number>(0);

  return (
    <div>
      <h1>Test</h1>
    </div>
  );
}
