export type ProductCategory =
  | 'Electronics'
  | 'Furniture'
  | 'Clothing'
  | 'Toys'
  | 'Groceries'
  | 'Books'
  | 'Jewelry'
  | 'Beauty Products';

export interface Product {
  id: number;
  name: string;
  description: string;
  price: number;
  photo_url: string;
  category: string;
  created_at: string;
  updated_at: string;
}
